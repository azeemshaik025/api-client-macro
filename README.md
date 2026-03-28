# API Client Macro

A procedural macro for generating type-safe API clients in Rust.

## Install

```sh
cargo add api-client-macro
```

You'll also need these dependencies:

```sh
cargo add reqwest --features json
cargo add serde --features derive
cargo add tokio --features full
```

## Usage

```rust
use api_client_macro::api_client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

#[derive(Serialize)]
struct UserPath {
    id: u32,
}

api_client!(
    UserApi,
    {
        {
            path: "/users",
            method: GET,
            res: Vec<User>,
        },
        {
            path: "/users/{id}",
            method: GET,
            path_params: UserPath,
            res: User,
        },
        {
            path: "/users",
            method: POST,
            req: User,
            res: User,
        },
    }
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = UserApi::new(
        reqwest::Url::parse("https://api.example.com")?,
        Some(5000),
    );

    let users = client.get_users().await?;
    let user = client.get_users_by_id(&UserPath { id: 1 }).await?;

    Ok(())
}
```

## Endpoint Fields

**Required:**

- `method`: HTTP method (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`)

**Optional:**

- `path`: URL path (e.g., `"/users/{id}"`)
- `res`: Response type (defaults to `()`)
- `req`: Request body type
- `path_params`: Type for path parameters
- `query_params`: Type for query parameters
- `headers`: Header type (e.g., `reqwest::header::HeaderMap`)
- `fn_name`: Custom function name
- `retries`: Retry count for this endpoint (overrides global)

## Auth

Add automatic authentication to every request. Three strategies are supported:

```rust
// Bearer token — injects `Authorization: Bearer <token>`
api_client!(GithubApi, auth: Bearer, { ... });
let client = GithubApi::new(url, "ghp_xxxx", Some(5000));

// Basic auth — injects `Authorization: Basic <base64>`
api_client!(DbApi, auth: Basic, { ... });
let client = DbApi::new(url, "admin", "secret", Some(5000));

// API key — injects a custom header
api_client!(StripeApi, auth: ApiKey("X-Api-Key"), { ... });
let client = StripeApi::new(url, "sk_live_xxxx", Some(5000));
```

Omitting `auth` keeps the original `new(url, timeout)` constructor (backward compatible). Auth works with all other features including retries.

## Retry with Backoff

Set a global retry count that applies to all endpoints. Retries use exponential backoff (100ms base, 2x multiplier, 5s cap) and trigger on 5xx errors and request timeouts. 4xx errors are never retried.

```rust
api_client!(
    UserApi,
    retries: 3,
    {
        {
            path: "/users",
            method: GET,
            res: Vec<User>,
            // inherits retries: 3 from global
        },
        {
            path: "/health",
            method: GET,
            retries: 0,  // override: no retries for this endpoint
        },
    }
);
```

Per-endpoint `retries` overrides the global value. Omitting `retries` entirely means no retries (backward compatible).

## Generated Code

The macro generates:

- A **struct** with `new(url, timeout)` constructor
- An **async method** for each endpoint
- A **trait** (`{Name}Trait`) for mocking in tests
- An **error enum** (`{Name}Error`) with variants for URL, request, HTTP, and deserialization errors

## Examples

See the [`examples/`](examples/) directory:

- [`basic.rs`](examples/basic.rs) - Simple GET requests
- [`params.rs`](examples/params.rs) - Path and query parameters
- [`advanced.rs`](examples/advanced.rs) - All features
- [`mocking.rs`](examples/mocking.rs) - Testing with generated traits
- [`multiple_path_params.rs`](examples/multiple_path_params.rs) - Nested resources

## License

MIT OR Apache-2.0
