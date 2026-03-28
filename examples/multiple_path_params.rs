//! Example demonstrating how multiple path parameters are handled in function names.
//!
//! When you have multiple path parameters like `/users/{user_id}/posts/{post_id}`,
//! the generated function name will be: `get_users_by_user_id_posts_by_post_id`

use http_provider_macro::api_client;
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Post {
    id: u32,
    title: String,
    content: String,
}

// Path parameters struct - fields must match all `{param}` placeholders in the path
#[derive(Serialize)]
struct UserPostPathParams {
    user_id: u32,
    post_id: u32,
}

// Define an API client with multiple path parameters
api_client!(
    ApiClient,
    {
        {
            // Path: /users/{user_id}/posts/{post_id}
            // Generated function name: get_users_posts_by_user_id_and_post_id
            path: "/users/{user_id}/posts/{post_id}",
            method: GET,
            path_params: UserPostPathParams,
            res: Post,
        },
        {
            // Path: /users/{id}/comments/{comment_id}/replies/{reply_id}
            // Generated function name: get_users_comments_replies_by_id_and_comment_id_and_reply_id
            path: "/users/{id}/comments/{comment_id}/replies/{reply_id}",
            method: GET,
            path_params: CommentReplyPathParams,
            res: String,
        },
    }
);

#[derive(Serialize)]
struct CommentReplyPathParams {
    id: u32,
    comment_id: u32,
    reply_id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = Url::parse("https://api.example.com")?;
    let client = ApiClient::new(base_url, Some(5000));

    // Use the generated method with multiple path parameters
    // Function name: get_users_posts_by_user_id_and_post_id
    let post = client
        .get_users_posts_by_user_id_and_post_id(&UserPostPathParams {
            user_id: 42,
            post_id: 100,
        })
        .await?;
    println!("Post: {:?}", post);

    // Another example with three path parameters
    // Function name: get_users_comments_replies_by_id_and_comment_id_and_reply_id
    let reply = client
        .get_users_comments_replies_by_id_and_comment_id_and_reply_id(&CommentReplyPathParams {
            id: 42,
            comment_id: 5,
            reply_id: 10,
        })
        .await?;
    println!("Reply: {}", reply);

    Ok(())
}
