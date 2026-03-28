//! Generate HTTP client methods from endpoint definitions.
//!
//! ```ignore
//! use http_provider_macro::api_client;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//! }
//!
//! api_client!(
//!     UserApi,
//!     {
//!         {
//!             path: "/users",
//!             method: GET,
//!             res: Vec<User>,
//!         },
//!         {
//!             path: "/users/{id}",
//!             method: GET,
//!             path_params: UserPath,
//!             res: User,
//!         }
//!     }
//! );
//!
//! #[derive(Serialize)]
//! struct UserPath {
//!     id: u32,
//! }
//!
//! // Usage
//! let client = UserApi::new(reqwest::Url::parse("https://api.example.com")?, Some(30));
//! let users = client.get_users().await?;
//! let user = client.get_users_by_id(&UserPath { id: 1 }).await?;
//! ```

extern crate proc_macro;

use crate::expanders::ApiClientExpander;
use crate::input::ApiClientInput;
use syn::parse_macro_input;

mod error;
mod expanders;
mod input;

fn expand_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ApiClientInput);
    match ApiClientExpander::new(input).expand() {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn api_client(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_macro(input)
}

#[proc_macro]
pub fn http_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_macro(input)
}
