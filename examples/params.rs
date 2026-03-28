//! Example demonstrating path parameters and query parameters.
//!
//! This example shows how to use `path_params` for dynamic URL segments
//! and `query_params` for query string parameters.

use api_client_macro::api_client;
use reqwest::Url;
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
struct SearchResults {
    results: Vec<User>,
    total: u32,
}

// Path parameters - fields must match the `{param}` placeholders in the path
#[derive(Serialize)]
struct UserPathParams {
    id: u32,
}

#[derive(Serialize)]
struct PostPathParams {
    post_id: u32,
}

// Query parameters - will be serialized as query string
#[derive(Serialize)]
struct SearchQueryParams {
    q: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

// Define an API client with path and query parameters
api_client!(
    ApiClient,
    {
        {
            path: "/users/{id}",
            method: GET,
            path_params: UserPathParams,
            res: User,
        },
        {
            path: "/posts/{post_id}",
            method: GET,
            path_params: PostPathParams,
            res: Post,
        },
        {
            path: "/search",
            method: GET,
            query_params: SearchQueryParams,
            res: SearchResults,
        },
    }
);

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Post {
    id: u32,
    title: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = Url::parse("https://api.example.com")?;
    let client = ApiClient::new(base_url, Some(5000));

    // Use path parameters - the `{id}` in the path will be replaced with the value
    let user = client.get_users_by_id(&UserPathParams { id: 42 }).await?;
    println!("User: {:?}", user);

    let post = client
        .get_posts_by_post_id(&PostPathParams { post_id: 1 })
        .await?;
    println!("Post: {:?}", post);

    // Use query parameters - will be serialized as ?q=rust&limit=10
    let results = client
        .get_search(&SearchQueryParams {
            q: "rust".to_string(),
            limit: Some(10),
            offset: Some(0),
        })
        .await?;
    println!("Search results: {:?}", results);

    Ok(())
}
