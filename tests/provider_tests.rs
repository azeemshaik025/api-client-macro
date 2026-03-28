#[cfg(test)]
mod tests {
    use api_client_macro::api_client;
    use reqwest::{header::HeaderMap, Url};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    // Define the client with various endpoint configurations
    api_client!(
        HttpProvider,
        {
            {
                path: "/users",
                method: GET,
                res: MyResponse,
            },
            {
                path: "/users/{id}",
                method: GET,
                path_params: PathParams,
                res: MyResponse,
            },
            {
                path: "/search",
                method: GET,
                query_params: QueryParams,
                res: MyResponse,
            },
            {
                path: "/data",
                method: GET,
                headers: HeaderMap,
                res: MyResponse,
            },
            {
                method: GET,
                res: MyResponse,
            },
            {
                path: "/users",
                method: POST,
                req: MyRequest,
                res: MyResponse,
            },
        }
    );

    // Test data structures
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct MyResponse {
        value: String,
    }

    #[derive(Serialize, Deserialize)]
    struct MyRequest {
        data: String,
    }

    #[derive(Serialize, Deserialize)]
    struct PathParams {
        id: String,
    }

    #[derive(Serialize, Deserialize)]
    struct QueryParams {
        q: String,
    }

    fn create_success_response(value: &str) -> MyResponse {
        MyResponse {
            value: value.to_string(),
        }
    }

    // Basic functionality tests
    #[tokio::test]
    async fn test_get_with_path() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let response = create_success_response("users");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = HttpProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider.get_users().await?;

        assert_eq!(result.value, "users");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_with_path_params() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let response = create_success_response("user-123");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path_regex(r"^/users/\w+$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = HttpProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider
            .get_users_by_id(&PathParams {
                id: "123".to_string(),
            })
            .await?;

        assert_eq!(result.value, "user-123");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_with_query_params() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let response = create_success_response("search-result");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/search"))
            .and(wiremock::matchers::query_param("q", "test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = HttpProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider
            .get_search(&QueryParams {
                q: "test".to_string(),
            })
            .await?;

        assert_eq!(result.value, "search-result");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_with_headers() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let response = create_success_response("with-headers");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/data"))
            .and(wiremock::matchers::header("x-api-key", "secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = HttpProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "secret".parse()?);

        let result = provider.get_data(headers).await?;

        assert_eq!(result.value, "with-headers");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_without_path() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let response = create_success_response("no-path");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = HttpProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider.get().await?;

        assert_eq!(result.value, "no-path");
        Ok(())
    }

    #[tokio::test]
    async fn test_post_with_body() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let response = create_success_response("created");

        Mock::given(method("POST"))
            .and(wiremock::matchers::path("/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = HttpProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider
            .post_users(&MyRequest {
                data: "test".to_string(),
            })
            .await?;

        assert_eq!(result.value, "created");
        Ok(())
    }

    // Trait-based mock client test
    #[tokio::test]
    async fn test_trait_mock_provider() -> Result<(), Box<dyn std::error::Error>> {
        // Simple client for trait testing
        api_client!(
            SimpleProvider,
            {
                {
                    path: "/items",
                    method: GET,
                    res: MyResponse,
                },
                {
                    path: "/items/{id}",
                    method: GET,
                    path_params: PathParams,
                    res: MyResponse,
                },
            }
        );

        struct MockProvider;

        impl SimpleProviderTrait for MockProvider {
            async fn get_items(&self) -> Result<MyResponse, SimpleProviderError> {
                Ok(create_success_response("mock-items"))
            }

            async fn get_items_by_id(
                &self,
                _path_params: &PathParams,
            ) -> Result<MyResponse, SimpleProviderError> {
                Ok(create_success_response("mock-item-123"))
            }
        }

        let mock = MockProvider;

        assert_eq!(mock.get_items().await?.value, "mock-items");
        assert_eq!(
            mock.get_items_by_id(&PathParams {
                id: "123".to_string()
            })
            .await?
            .value,
            "mock-item-123"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_patch_with_body() -> Result<(), Box<dyn std::error::Error>> {
        api_client!(
            PatchProvider,
            {
                {
                    path: "/users/{id}",
                    method: PATCH,
                    path_params: PathParams,
                    req: MyRequest,
                    res: MyResponse,
                },
            }
        );

        let mock_server = MockServer::start().await;
        let response = create_success_response("patched");

        Mock::given(method("PATCH"))
            .and(wiremock::matchers::path_regex(r"^/users/\w+$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = PatchProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider
            .patch_users_by_id(
                &PathParams {
                    id: "42".to_string(),
                },
                &MyRequest {
                    data: "partial".to_string(),
                },
            )
            .await?;

        assert_eq!(result.value, "patched");
        Ok(())
    }

    #[tokio::test]
    async fn test_http_error_response() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/users"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let provider = HttpProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider.get_users().await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("404"),
            "Expected 404 in error: {}",
            err_msg
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_path_params() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Serialize, Deserialize)]
        struct MultiPathParams {
            user_id: String,
            post_id: String,
        }

        api_client!(
            MultiParamProvider,
            {
                {
                    path: "/users/{user_id}/posts/{post_id}",
                    method: GET,
                    path_params: MultiPathParams,
                    res: MyResponse,
                },
            }
        );

        let mock_server = MockServer::start().await;
        let response = create_success_response("nested");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/users/1/posts/2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = MultiParamProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider
            .get_users_posts_by_user_id_and_post_id(&MultiPathParams {
                user_id: "1".to_string(),
                post_id: "2".to_string(),
            })
            .await?;

        assert_eq!(result.value, "nested");
        Ok(())
    }

    // --- Retry tests ---

    #[tokio::test]
    async fn test_retry_on_5xx_succeeds_after_transient_failure(
    ) -> Result<(), Box<dyn std::error::Error>> {
        api_client!(
            RetryProvider,
            retries: 3,
            {
                {
                    path: "/flaky",
                    method: GET,
                    res: MyResponse,
                },
            }
        );

        let mock_server = MockServer::start().await;
        let response = create_success_response("recovered");

        // Base mock: returns 200 (lower priority)
        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/flaky"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        // Override mock: returns 500, only first 2 times (higher priority)
        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/flaky"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        let provider = RetryProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider.get_flaky().await?;

        assert_eq!(result.value, "recovered");
        Ok(())
    }

    #[tokio::test]
    async fn test_no_retry_on_4xx() -> Result<(), Box<dyn std::error::Error>> {
        api_client!(
            Retry4xxProvider,
            retries: 3,
            {
                {
                    path: "/not-found",
                    method: GET,
                    res: MyResponse,
                },
            }
        );

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/not-found"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&mock_server)
            .await;

        let provider = Retry4xxProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider.get_not_found().await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_retry_endpoint_override() -> Result<(), Box<dyn std::error::Error>> {
        api_client!(
            RetryOverrideProvider,
            retries: 3,
            {
                {
                    path: "/no-retry",
                    method: GET,
                    res: MyResponse,
                    retries: 0,
                },
            }
        );

        let mock_server = MockServer::start().await;

        // Returns 500 always — with retries: 0 override, should fail immediately
        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/no-retry"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let provider = RetryOverrideProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider.get_no_retry().await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backward_compat_no_retries_field() -> Result<(), Box<dyn std::error::Error>> {
        // Uses existing HttpProvider which has no retries field — should still work
        let mock_server = MockServer::start().await;
        let response = create_success_response("compat");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = HttpProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));
        let result = provider.get_users().await?;

        assert_eq!(result.value, "compat");
        Ok(())
    }

    #[tokio::test]
    async fn test_optional_response() -> Result<(), Box<dyn std::error::Error>> {
        // Client with optional response (no res field)
        api_client!(
            NoResponseProvider,
            {
                {
                    path: "/delete",
                    method: DELETE,
                },
                {
                    path: "/update",
                    method: PUT,
                    req: MyRequest,
                },
            }
        );

        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(wiremock::matchers::path("/delete"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        Mock::given(method("PUT"))
            .and(wiremock::matchers::path("/update"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let provider = NoResponseProvider::new(Url::from_str(&mock_server.uri())?, Some(5000));

        // Test DELETE without response
        let result: Result<(), _> = provider.delete_delete().await;
        assert!(result.is_ok());

        // Test PUT without response
        let result: Result<(), _> = provider
            .put_update(&MyRequest {
                data: "test".to_string(),
            })
            .await;
        assert!(result.is_ok());

        Ok(())
    }

    // --- Auth tests ---

    #[tokio::test]
    async fn test_bearer_auth() -> Result<(), Box<dyn std::error::Error>> {
        api_client!(
            BearerApi,
            auth: Bearer,
            {
                {
                    path: "/protected",
                    method: GET,
                    res: MyResponse,
                },
            }
        );

        let mock_server = MockServer::start().await;
        let response = create_success_response("bearer-ok");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/protected"))
            .and(wiremock::matchers::header(
                "Authorization",
                "Bearer my-token",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = BearerApi::new(Url::from_str(&mock_server.uri())?, "my-token", Some(5000));
        let result = provider.get_protected().await?;

        assert_eq!(result.value, "bearer-ok");
        Ok(())
    }

    #[tokio::test]
    async fn test_basic_auth() -> Result<(), Box<dyn std::error::Error>> {
        api_client!(
            BasicApi,
            auth: Basic,
            {
                {
                    path: "/secure",
                    method: GET,
                    res: MyResponse,
                },
            }
        );

        let mock_server = MockServer::start().await;
        let response = create_success_response("basic-ok");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/secure"))
            .and(wiremock::matchers::header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = BasicApi::new(
            Url::from_str(&mock_server.uri())?,
            "user",
            "pass",
            Some(5000),
        );
        let result = provider.get_secure().await?;

        assert_eq!(result.value, "basic-ok");
        Ok(())
    }

    #[tokio::test]
    async fn test_api_key_auth() -> Result<(), Box<dyn std::error::Error>> {
        api_client!(
            ApiKeyApi,
            auth: ApiKey("X-Api-Key"),
            {
                {
                    path: "/data",
                    method: GET,
                    res: MyResponse,
                },
            }
        );

        let mock_server = MockServer::start().await;
        let response = create_success_response("apikey-ok");

        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/data"))
            .and(wiremock::matchers::header("X-Api-Key", "sk_live_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let provider = ApiKeyApi::new(
            Url::from_str(&mock_server.uri())?,
            "sk_live_123",
            Some(5000),
        );
        let result = provider.get_data().await?;

        assert_eq!(result.value, "apikey-ok");
        Ok(())
    }

    #[tokio::test]
    async fn test_auth_with_retries() -> Result<(), Box<dyn std::error::Error>> {
        api_client!(
            AuthRetryApi,
            auth: Bearer,
            retries: 2,
            {
                {
                    path: "/flaky-auth",
                    method: GET,
                    res: MyResponse,
                },
            }
        );

        let mock_server = MockServer::start().await;
        let response = create_success_response("auth-retry-ok");

        // Base mock: returns 200 with auth check
        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/flaky-auth"))
            .and(wiremock::matchers::header("Authorization", "Bearer tok"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        // Override: 500 once
        Mock::given(method("GET"))
            .and(wiremock::matchers::path("/flaky-auth"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let provider = AuthRetryApi::new(Url::from_str(&mock_server.uri())?, "tok", Some(5000));
        let result = provider.get_flaky_auth().await?;

        assert_eq!(result.value, "auth-retry-ok");
        Ok(())
    }
}
