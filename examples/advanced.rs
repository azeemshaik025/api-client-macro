//! Comprehensive example demonstrating all features of the `api_client` macro.
//!
//! This example shows:
//! - Request bodies (`req`)
//! - Custom headers (`headers`)
//! - Query parameters (`query_params`)
//! - Path parameters (`path_params`)
//! - Custom function names (`fn_name`)
//! - Optional response types (omitting `res` returns `()`)
//! - Endpoints without paths

use http_provider_macro::api_client;
use reqwest::{header::HeaderMap, Url};
use serde::{Deserialize, Serialize};

// Response types
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CreateUserResponse {
    id: u32,
    message: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SearchResults {
    results: Vec<User>,
    total: u32,
}

// Request body types
#[derive(Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

// Path parameters
#[derive(Serialize)]
struct UserPathParams {
    id: u32,
}

// Query parameters
#[derive(Serialize)]
struct SearchQueryParams {
    q: String,
    limit: Option<u32>,
}

// Define an API client with all features
api_client!(
    ApiClient,
    {
        // Basic GET with path and response
        {
            path: "/users",
            method: GET,
            res: Vec<User>,
        },
        // GET with path parameters
        {
            path: "/users/{id}",
            method: GET,
            path_params: UserPathParams,
            res: User,
        },
        // GET with query parameters
        {
            path: "/search",
            method: GET,
            query_params: SearchQueryParams,
            res: SearchResults,
        },
        // GET with custom headers
        {
            path: "/protected/data",
            method: GET,
            headers: HeaderMap,
            res: User,
        },
        // POST with request body
        {
            path: "/users",
            method: POST,
            req: CreateUserRequest,
            res: CreateUserResponse,
        },
        // PUT with path params and request body
        {
            path: "/users/{id}",
            method: PUT,
            path_params: UserPathParams,
            req: UpdateUserRequest,
            res: User,
        },
        // DELETE with path params and no response body
        {
            path: "/users/{id}",
            method: DELETE,
            path_params: UserPathParams,
        },
        // Custom function name
        {
            path: "/users/me",
            method: GET,
            fn_name: get_current_user,
            res: User,
        },
        // Endpoint without path (uses base URL root)
        {
            method: GET,
            res: User,
        },
    }
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = Url::parse("https://api.example.com")?;
    let client = ApiClient::new(base_url, Some(5000));

    // Basic GET request
    let users = client.get_users().await?;
    println!("Users: {:?}", users);

    // GET with path parameters
    let user = client.get_users_by_id(&UserPathParams { id: 42 }).await?;
    println!("User: {:?}", user);

    // GET with query parameters
    let results = client
        .get_search(&SearchQueryParams {
            q: "rust".to_string(),
            limit: Some(10),
        })
        .await?;
    println!("Search results: {:?}", results);

    // GET with custom headers
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", "Bearer token123".parse()?);
    let protected_data = client.get_protected_data(headers).await?;
    println!("Protected data: {:?}", protected_data);

    // POST with request body
    let new_user = client
        .post_users(&CreateUserRequest {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        })
        .await?;
    println!("Created user: {:?}", new_user);

    // PUT with path params and request body
    let updated_user = client
        .put_users_by_id(
            &UserPathParams { id: 42 },
            &UpdateUserRequest {
                name: Some("Jane Doe".to_string()),
                email: None,
            },
        )
        .await?;
    println!("Updated user: {:?}", updated_user);

    // DELETE with path params (no response body)
    client
        .delete_users_by_id(&UserPathParams { id: 42 })
        .await?;
    println!("User deleted");

    // Custom function name
    let current_user = client.get_current_user().await?;
    println!("Current user: {:?}", current_user);

    // Endpoint without path
    let root_data = client.get().await?;
    println!("Root data: {:?}", root_data);

    Ok(())
}
