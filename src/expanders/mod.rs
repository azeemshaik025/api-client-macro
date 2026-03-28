use crate::{
    error::{MacroError, MacroResult},
    input::{ApiClientInput, AuthStrategy},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub mod error;
pub mod interface;
pub mod method;

pub use error::ErrorExpander;
pub use interface::TraitExpander;
pub use method::MethodExpander;

pub struct ApiClientExpander {
    input: ApiClientInput,
}

impl ApiClientExpander {
    pub fn new(input: ApiClientInput) -> Self {
        Self { input }
    }

    pub fn expand(&self) -> MacroResult<TokenStream> {
        self.validate()?;

        let struct_name = &self.input.struct_name;
        let error_name = Ident::new(&format!("{}Error", struct_name), struct_name.span());

        let error_type = ErrorExpander::new(&error_name).expand();
        let trait_def = self.expand_trait_def(&error_name)?;
        let methods = self.expand_methods(&error_name)?;
        let struct_impl = self.expand_struct_impl(&methods);

        Ok(quote! {
            #error_type
            #trait_def
            #struct_impl
        })
    }

    fn expand_trait_def(&self, error_name: &Ident) -> MacroResult<TokenStream> {
        let trait_name = self.trait_name();
        TraitExpander::new(&self.input.endpoints, &trait_name, error_name).expand()
    }

    fn expand_methods(&self, error_name: &Ident) -> MacroResult<Vec<TokenStream>> {
        let global_retries = self.input.global_retries.unwrap_or(0);
        let auth = self.input.auth.as_ref();
        self.input
            .endpoints
            .iter()
            .map(|def| {
                let retry_count = def.retries.unwrap_or(global_retries);
                MethodExpander::new(def, error_name, retry_count, auth).expand()
            })
            .collect()
    }

    fn expand_struct_impl(&self, methods: &[TokenStream]) -> TokenStream {
        let struct_name = &self.input.struct_name;
        let trait_name = self.trait_name();

        let (auth_fields, auth_params, auth_inits) = match &self.input.auth {
            Some(AuthStrategy::Bearer) => (
                quote! { token: String, },
                quote! { token: &str, },
                quote! { token: token.to_string(), },
            ),
            Some(AuthStrategy::Basic) => (
                quote! { username: String, password: String, },
                quote! { username: &str, password: &str, },
                quote! { username: username.to_string(), password: password.to_string(), },
            ),
            Some(AuthStrategy::ApiKey(_)) => (
                quote! { api_key: String, },
                quote! { api_key: &str, },
                quote! { api_key: api_key.to_string(), },
            ),
            None => (quote! {}, quote! {}, quote! {}),
        };

        quote! {
            pub struct #struct_name {
                url: reqwest::Url,
                client: reqwest::Client,
                timeout: std::time::Duration,
                #auth_fields
            }

            impl #struct_name {
                pub fn new(url: reqwest::Url, #auth_params timeout: Option<u64>) -> Self {
                    let client = reqwest::Client::new();
                    let timeout = std::time::Duration::from_millis(timeout.unwrap_or(5000));
                    Self { url, client, timeout, #auth_inits }
                }
            }

            impl #trait_name for #struct_name {
                #(#methods)*
            }
        }
    }

    fn trait_name(&self) -> Ident {
        Ident::new(
            &format!("{}Trait", self.input.struct_name),
            self.input.struct_name.span(),
        )
    }

    fn validate(&self) -> MacroResult<()> {
        if self.input.endpoints.is_empty() {
            return Err(MacroError::NoEndpointsConfigured {
                span: self.input.struct_name.span(),
            });
        }
        Ok(())
    }
}
