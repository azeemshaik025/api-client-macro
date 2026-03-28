use crate::{
    error::MacroResult,
    input::{AuthStrategy, EndpointDef, HttpMethod},
};
use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::Ident;

const PATH_PARAM_REGEX: &str = r"\{([a-zA-Z0-9_]+)\}";

pub struct MethodExpander<'a> {
    def: &'a EndpointDef,
    error_name: &'a Ident,
    retry_count: u32,
    auth: Option<&'a AuthStrategy>,
}

impl<'a> MethodExpander<'a> {
    pub fn new(
        def: &'a EndpointDef,
        error_name: &'a Ident,
        retry_count: u32,
        auth: Option<&'a AuthStrategy>,
    ) -> Self {
        Self {
            def,
            error_name,
            retry_count,
            auth,
        }
    }

    pub fn expand(&self) -> MacroResult<TokenStream> {
        let fn_name = FnNameExpander::new(self.def).expand();
        let params = ParamsExpander::new(self.def).expand();
        let res = self
            .def
            .res
            .as_ref()
            .map(|t| quote! { #t })
            .unwrap_or_else(|| quote! { () });
        let error_name = self.error_name;

        let url_construction = UrlExpander::new(self.def, self.error_name).expand();
        let request_builder = RequestExpander::new(self.def, self.auth).expand();
        let deserialize = ResponseExpander::new(self.def.res.as_ref(), self.error_name).expand();

        let body = if self.retry_count > 0 {
            let max_retries = self.retry_count;
            quote! {
                #url_construction
                let mut __attempt: u32 = 0;
                loop {
                    #request_builder
                    let __response = request.send().await;
                    match __response {
                        Err(err) => {
                            if __attempt < #max_retries && err.is_timeout() {
                                __attempt += 1;
                                let __delay = std::cmp::min(
                                    100u64 * (1u64 << __attempt),
                                    5000u64,
                                );
                                tokio::time::sleep(
                                    std::time::Duration::from_millis(__delay),
                                )
                                .await;
                                continue;
                            }
                            return Err(#error_name::from(err));
                        }
                        Ok(response) => {
                            let status = response.status();
                            if !status.is_success() {
                                if __attempt < #max_retries && status.is_server_error() {
                                    __attempt += 1;
                                    let __delay = std::cmp::min(
                                        100u64 * (1u64 << __attempt),
                                        5000u64,
                                    );
                                    tokio::time::sleep(
                                        std::time::Duration::from_millis(__delay),
                                    )
                                    .await;
                                    continue;
                                }
                                let reason = status
                                    .canonical_reason()
                                    .unwrap_or("Unknown")
                                    .to_string();
                                return Err(#error_name::Http {
                                    status: status.as_u16(),
                                    reason,
                                });
                            }
                            break #deserialize;
                        }
                    }
                }
            }
        } else {
            quote! {
                #url_construction
                #request_builder
                let response = request.send().await.map_err(#error_name::from)?;
                let status = response.status();
                if !status.is_success() {
                    let reason = status
                        .canonical_reason()
                        .unwrap_or("Unknown")
                        .to_string();
                    return Err(#error_name::Http {
                        status: status.as_u16(),
                        reason,
                    });
                }
                #deserialize
            }
        };

        Ok(quote! {
            async fn #fn_name(&self, #(#params),*) -> Result<#res, #error_name> {
                #body
            }
        })
    }
}

pub struct FnNameExpander<'a> {
    def: &'a EndpointDef,
}

impl<'a> FnNameExpander<'a> {
    pub fn new(def: &'a EndpointDef) -> Self {
        Self { def }
    }

    pub fn expand(&self) -> Ident {
        if let Some(ref name) = self.def.fn_name {
            return name.clone();
        }

        let method_str = self.def.method.as_str();
        let name = if let Some(ref path) = self.def.path {
            let path_str = path.value().trim_start_matches('/').to_string();
            let path_part = self.expand_fn_name_with_path(&path_str);
            format!("{}_{}", method_str, path_part).to_snake_case()
        } else {
            method_str.to_string()
        };

        Ident::new(
            &name,
            self.def
                .path
                .as_ref()
                .map_or_else(Span::call_site, |p| p.span()),
        )
    }

    fn expand_fn_name_with_path(&self, path_str: &str) -> String {
        // Handle path parameters: extract them and format as by_{param1}_and_{param2}...
        if self.def.path_params.is_some() {
            let re = Regex::new(PATH_PARAM_REGEX).expect("Invalid regex");
            let mut param_names: Vec<String> = Vec::new();
            let mut base_path = path_str.to_string();

            // Extract all parameter names
            for cap in re.captures_iter(path_str) {
                param_names.push(cap[1].to_string());
            }

            // Remove path parameters from the base path
            base_path = re.replace_all(&base_path, "").to_string();
            // Clean up double slashes and trailing slashes
            base_path = base_path.replace("//", "/").trim_matches('/').to_string();

            // Build the path part of the function name
            let base_part = base_path.replace("/", "_");
            if !param_names.is_empty() {
                let params_part = if param_names.len() == 1 {
                    format!("by_{}", param_names[0])
                } else {
                    format!("by_{}", param_names.join("_and_"))
                };

                if !base_part.is_empty() {
                    format!("{}_{}", base_part, params_part)
                } else {
                    params_part
                }
            } else {
                base_part
            }
        } else {
            // No path parameters, just use the path as-is
            path_str.replace("/", "_")
        }
    }
}

pub struct ParamsExpander<'a> {
    def: &'a EndpointDef,
}

impl<'a> ParamsExpander<'a> {
    pub fn new(def: &'a EndpointDef) -> Self {
        Self { def }
    }

    pub fn expand(&self) -> Vec<TokenStream> {
        let mut params = Vec::new();

        if let Some(ref path_params) = self.def.path_params {
            params.push(quote! { path_params: &#path_params });
        }
        if let Some(ref body) = self.def.req {
            params.push(quote! { body: &#body });
        }
        if let Some(ref query_params) = self.def.query_params {
            params.push(quote! { query_params: &#query_params });
        }
        if let Some(ref headers) = self.def.headers {
            params.push(quote! { headers: #headers });
        }

        params
    }
}

pub struct UrlExpander<'a> {
    def: &'a EndpointDef,
    error_name: &'a Ident,
}

impl<'a> UrlExpander<'a> {
    pub fn new(def: &'a EndpointDef, error_name: &'a Ident) -> Self {
        Self { def, error_name }
    }

    pub fn expand(&self) -> TokenStream {
        let Some(ref path) = self.def.path else {
            return quote! { let url = self.url.clone(); };
        };

        if self.def.path_params.is_some() {
            self.expand_with_path_params(path)
        } else {
            self.expand_without_path_params(path)
        }
    }

    fn expand_with_path_params(&self, path: &syn::LitStr) -> TokenStream {
        let re = Regex::new(PATH_PARAM_REGEX).expect("Invalid regex");
        let path_str = path.value();
        let replacements: Vec<_> = re
            .captures_iter(&path_str)
            .map(|cap| {
                let param_name = &cap[1];
                let ident = Ident::new(param_name, Span::call_site());
                quote! {
                    path = path.replace(concat!("{", #param_name, "}"), &path_params.#ident.to_string());
                }
            })
            .collect();

        let error_name = self.error_name;
        quote! {
            let mut path = #path.to_string();
            #(#replacements)*
            let url = self.url.join(&path)
                .map_err(|e| #error_name::UrlConstruction(e.to_string()))?;
        }
    }

    fn expand_without_path_params(&self, path: &syn::LitStr) -> TokenStream {
        let error_name = self.error_name;
        quote! {
            let url = self.url.join(#path)
                .map_err(|e| #error_name::UrlConstruction(e.to_string()))?;
        }
    }
}

pub struct RequestExpander<'a> {
    def: &'a EndpointDef,
    auth: Option<&'a AuthStrategy>,
}

impl<'a> RequestExpander<'a> {
    pub fn new(def: &'a EndpointDef, auth: Option<&'a AuthStrategy>) -> Self {
        Self { def, auth }
    }

    pub fn expand(&self) -> TokenStream {
        let method_call = self.expand_method_call();
        let modifications = self.expand_modifications();

        quote! {
            let mut request = #method_call.timeout(self.timeout);
            #(#modifications)*
        }
    }

    fn expand_method_call(&self) -> TokenStream {
        match self.def.method {
            HttpMethod::GET => quote! { self.client.get(url.clone()) },
            HttpMethod::POST => quote! { self.client.post(url.clone()) },
            HttpMethod::PUT => quote! { self.client.put(url.clone()) },
            HttpMethod::DELETE => quote! { self.client.delete(url.clone()) },
            HttpMethod::PATCH => quote! { self.client.patch(url.clone()) },
        }
    }

    fn expand_modifications(&self) -> Vec<TokenStream> {
        let mut modifications = Vec::new();

        match self.auth {
            Some(AuthStrategy::Bearer) => {
                modifications.push(quote! { request = request.bearer_auth(&self.token); });
            }
            Some(AuthStrategy::Basic) => {
                modifications.push(
                    quote! { request = request.basic_auth(&self.username, Some(&self.password)); },
                );
            }
            Some(AuthStrategy::ApiKey(ref header_name)) => {
                modifications
                    .push(quote! { request = request.header(#header_name, &self.api_key); });
            }
            None => {}
        }

        if self.def.req.is_some() {
            modifications.push(quote! { request = request.json(body); });
        }
        if self.def.query_params.is_some() {
            modifications.push(quote! { request = request.query(query_params); });
        }
        if self.def.headers.is_some() {
            modifications.push(quote! { request = request.headers(headers.clone()); });
        }

        modifications
    }
}

pub struct ResponseExpander<'a> {
    res: Option<&'a syn::Type>,
    error_name: &'a Ident,
}

impl<'a> ResponseExpander<'a> {
    pub fn new(res: Option<&'a syn::Type>, error_name: &'a Ident) -> Self {
        Self { res, error_name }
    }

    pub fn expand(&self) -> TokenStream {
        let error_name = self.error_name;
        match self.res {
            Some(res) => quote! {
                response
                    .json::<#res>()
                    .await
                    .map_err(|e| #error_name::Deserialization(e.to_string()))
            },
            None => quote! {
                Ok(())
            },
        }
    }
}
