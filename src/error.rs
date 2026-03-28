use proc_macro2::Span;
use syn::Error as SynError;

/// Custom error types for the API client macro.
///
/// This enum represents the different types of errors that can occur
/// during macro expansion and code generation.
#[derive(Debug)]
pub enum MacroError {
    Syn(SynError),
    NoEndpointsConfigured { span: Span },
}

impl MacroError {
    /// Converts the error into a token stream that can be used in compile-time error reporting.
    ///
    /// This method ensures that errors are properly displayed in the Rust compiler's
    /// error messages with appropriate source code locations.
    ///
    /// # Returns
    /// * `proc_macro2::TokenStream` - A token stream representing the error message
    pub fn into_compile_error(self) -> proc_macro2::TokenStream {
        match self {
            MacroError::Syn(err) => err.to_compile_error(),
            MacroError::NoEndpointsConfigured { span } => {
                SynError::new(span, "at least one endpoint must be defined").to_compile_error()
            }
        }
    }
}

impl From<SynError> for MacroError {
    fn from(err: SynError) -> Self {
        MacroError::Syn(err)
    }
}

pub type MacroResult<T> = std::result::Result<T, MacroError>;
