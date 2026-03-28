use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, LitInt, LitStr, Token, Type,
};

/// Represents HTTP methods supported by the API client macro.
///
/// These methods align with standard HTTP/1.1 methods and are used
/// to define the type of request for each endpoint.
#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    /// HTTP GET method for retrieving resources
    GET,

    /// HTTP POST method for creating resources
    POST,

    /// HTTP PUT method for updating resources
    PUT,

    /// HTTP DELETE method for removing resources
    DELETE,

    /// HTTP PATCH method for partially updating resources
    PATCH,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "get",
            HttpMethod::POST => "post",
            HttpMethod::PUT => "put",
            HttpMethod::DELETE => "delete",
            HttpMethod::PATCH => "patch",
        }
    }
}

impl Parse for HttpMethod {
    /// Parses an HTTP method from the input stream.
    ///
    /// # Arguments
    /// * `input` - The parse stream containing the method identifier
    ///
    /// # Returns
    /// * `Result<Self>` - The parsed HTTP method or an error if method is unsupported
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unsupported HTTP method: {}", ident),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthStrategy {
    Bearer,
    Basic,
    ApiKey(String),
}

impl Parse for AuthStrategy {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "Bearer" => Ok(AuthStrategy::Bearer),
            "Basic" => Ok(AuthStrategy::Basic),
            "ApiKey" => {
                let content;
                parenthesized!(content in input);
                let header: LitStr = content.parse()?;
                Ok(AuthStrategy::ApiKey(header.value()))
            }
            _ => Err(syn::Error::new(
                ident.span(),
                format!(
                    "Unsupported auth strategy: {}. Expected Bearer, Basic, or ApiKey(\"header\")",
                    ident
                ),
            )),
        }
    }
}

/// Root structure for parsing the API client macro input.
///
/// This structure represents the complete macro definition including
/// the generated client struct name and all its endpoint definitions.
///
/// # Example
/// ```ignore
/// MyApiClient, {
///     {
///         path: "/users",
///         method: GET,
///         res: Vec<User>
///     }
/// }
/// ```
pub struct ApiClientInput {
    /// Name of the client struct that will be generated
    pub struct_name: Ident,

    /// Optional auth strategy for all endpoints
    pub auth: Option<AuthStrategy>,

    /// Optional global retry count for all endpoints
    pub global_retries: Option<u32>,

    /// Collection of endpoint definitions
    pub endpoints: Vec<EndpointDef>,
}

/// Represents a single API endpoint configuration, ordered by importance.
///
/// The order below reflects the typical essential elements of an API endpoint:
/// * `method` - The HTTP method to use (required)
/// * `res` - Response type that will be deserialized (optional, defaults to `()`)
/// * `path` - Optional URL path for the endpoint (e.g., "/api/users")
/// * `fn_name` - Optional custom name for the generated function
/// * `req` - Optional request body type
/// * `headers` - Optional custom headers type
/// * `query_params` - Optional query parameters type
/// * `path_params` - Optional path parameters type
pub struct EndpointDef {
    pub method: HttpMethod,
    pub res: Option<Type>,

    pub path: Option<LitStr>,
    pub fn_name: Option<Ident>,
    pub req: Option<Type>,
    pub headers: Option<Type>,
    pub query_params: Option<Type>,
    pub path_params: Option<Type>,
    pub retries: Option<u32>,
}

impl Parse for ApiClientInput {
    /// Parses the complete macro input into a structured form.
    ///
    /// Expects input in the format:
    /// `struct_name, { endpoint1, endpoint2, ... }`
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse optional global options: `auth: Strategy,` and `retries: N,`
        let mut auth = None;
        let mut global_retries = None;

        while input.peek(Ident) && input.peek2(Token![:]) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            match ident.to_string().as_str() {
                "auth" => {
                    let _: Ident = input.parse()?;
                    input.parse::<Token![:]>()?;
                    auth = Some(input.parse::<AuthStrategy>()?);
                    input.parse::<Token![,]>()?;
                }
                "retries" => {
                    let _: Ident = input.parse()?;
                    input.parse::<Token![:]>()?;
                    let lit: LitInt = input.parse()?;
                    global_retries = Some(lit.base10_parse::<u32>()?);
                    input.parse::<Token![,]>()?;
                }
                _ => break,
            }
        }

        let content;
        braced!(content in input);
        let items: Punctuated<EndpointDef, Token![,]> =
            content.parse_terminated(EndpointDef::parse, Token![,])?;

        Ok(Self {
            struct_name,
            auth,
            global_retries,
            endpoints: items.into_iter().collect(),
        })
    }
}

impl Parse for EndpointDef {
    /// Parses a single endpoint definition block.
    ///
    /// # Format
    /// ```ignore
    /// {
    ///     path: "/path",
    ///     method: GET,
    ///     fn_name: custom_name,      // optional
    ///     req: RequestType,          // optional
    ///     res: ResponseType,         // optional, defaults to () if omitted
    ///     headers: HeadersType,      // optional
    ///     query_params: QueryType,   // optional
    ///     path_params: ParamsType    // optional
    /// }
    /// ```
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);

        let mut path = None;
        let mut method = None;
        let mut fn_name = None;
        let mut req = None;
        let mut res = None;
        let mut headers = None;
        let mut query_params = None;
        let mut path_params = None;
        let mut retries = None;

        // Iteratively parse each key-value pair inside the endpoint block
        while !content.is_empty() {
            let field: Ident = content.parse()?;
            content.parse::<Token![:]>()?;

            match field.to_string().as_str() {
                "path" => path = Some(content.parse()?),
                "method" => method = Some(content.parse()?),
                "fn_name" => fn_name = Some(content.parse()?),
                "req" => req = Some(content.parse()?),
                "res" => res = Some(content.parse()?),
                "headers" => headers = Some(content.parse()?),
                "query_params" => query_params = Some(content.parse()?),
                "path_params" => path_params = Some(content.parse()?),
                "retries" => {
                    let lit: LitInt = content.parse()?;
                    retries = Some(lit.base10_parse::<u32>()?);
                }
                _ => return Err(syn::Error::new(field.span(), "unexpected field")),
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(EndpointDef {
            path,
            method: method.ok_or_else(|| syn::Error::new(content.span(), "missing `method`"))?,
            fn_name,
            req,
            res,
            headers,
            query_params,
            path_params,
            retries,
        })
    }
}
