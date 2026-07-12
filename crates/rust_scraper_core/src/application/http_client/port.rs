//! Application-layer HTTP port trait.
//!
//! Defines [`HttpClientPort`] — the abstraction that application services depend
//! on for HTTP fetching. The real [`HttpClient`](super::HttpClient) and test
//! doubles both implement this trait, enabling mock-based unit tests without
//! network I/O.

use std::collections::HashMap;
use std::pin::Pin;

use super::error::{HttpError, HttpResult};

/// Simplified HTTP response for application-layer consumption.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code (e.g. 200, 404).
    pub status: u16,
    /// Response body as a UTF-8 string.
    pub body: String,
    /// Response headers (lowercased keys).
    pub headers: HashMap<String, String>,
}

/// Port trait for HTTP requests — application layer depends on this, not `wreq`.
///
/// Implementors provide the actual network I/O (production) or canned responses
/// (tests). This trait is intentionally thin — only `get` is required — so that
/// mock implementations stay simple and fast to compile.
///
/// # Thread safety
///
/// Implementations must be `Send + Sync` to work with Tokio's multi-threaded
/// runtime.
pub trait HttpClientPort: Send + Sync {
    /// Fetch a URL and return the response body.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] on network failure, timeout, or non-2xx status.
    fn get(
        &self,
        url: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = HttpResult<HttpResponse>> + Send + '_>>;
}

/// Production implementation of [`HttpClientPort`] backed by `wreq`.
///
/// Performs a single GET request with no retry logic. Retries and
/// user-agent rotation live in [`super::HttpClient`] — this impl is
/// intentionally thin so that the application layer stays decoupled
/// from retry policy.
impl HttpClientPort for wreq::Client {
    fn get(
        &self,
        url: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = HttpResult<HttpResponse>> + Send + '_>> {
        let url = url.to_owned();
        Box::pin(async move {
            let response = self.get(url.as_str()).send().await.map_err(|e| {
                if e.is_timeout() {
                    HttpError::Timeout
                } else if e.is_connect() {
                    HttpError::Connection(e.to_string())
                } else {
                    HttpError::Request(e.to_string())
                }
            })?;

            let status = response.status().as_u16();

            let mut headers = HashMap::new();
            for (key, value) in response.headers() {
                if let Ok(v) = value.to_str() {
                    headers.insert(key.as_str().to_lowercase(), v.to_string());
                }
            }

            let body = response
                .text()
                .await
                .map_err(|e| HttpError::Request(e.to_string()))?;

            Ok(HttpResponse {
                status,
                body,
                headers,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::error::HttpError;
    use super::*;
    use std::collections::HashMap;

    /// Minimal mock for verifying trait contract.
    struct MockHttpClient {
        responses: HashMap<String, HttpResult<HttpResponse>>,
    }

    impl MockHttpClient {
        fn new() -> Self {
            Self {
                responses: HashMap::new(),
            }
        }

        fn with_response(mut self, url: &str, result: HttpResult<HttpResponse>) -> Self {
            self.responses.insert(url.to_string(), result);
            self
        }
    }

    impl HttpClientPort for MockHttpClient {
        fn get(
            &self,
            url: &str,
        ) -> Pin<Box<dyn std::future::Future<Output = HttpResult<HttpResponse>> + Send + '_>>
        {
            let result = self
                .responses
                .get(url)
                .cloned()
                .unwrap_or(Err(HttpError::ClientError(404)));
            Box::pin(async move { result })
        }
    }

    #[tokio::test]
    async fn test_mock_returns_success() {
        let mock = MockHttpClient::new().with_response(
            "https://example.com",
            Ok(HttpResponse {
                status: 200,
                body: "<p>Hello</p>".into(),
                headers: HashMap::new(),
            }),
        );

        let resp = mock.get("https://example.com").await.unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "<p>Hello</p>");
    }

    #[tokio::test]
    async fn test_mock_returns_error_for_unknown_url() {
        let mock = MockHttpClient::new();
        let result = mock.get("https://unknown.com").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), HttpError::ClientError(404));
    }

    #[tokio::test]
    async fn test_mock_returns_timeout() {
        let mock = MockHttpClient::new().with_response("https://slow.com", Err(HttpError::Timeout));
        let result = mock.get("https://slow.com").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), HttpError::Timeout);
    }

    #[tokio::test]
    async fn test_mock_returns_rate_limited() {
        let mock =
            MockHttpClient::new().with_response("https://api.com", Err(HttpError::RateLimited(60)));
        let result = mock.get("https://api.com").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), HttpError::RateLimited(60));
    }

    #[tokio::test]
    async fn test_mock_response_headers() {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "text/html".into());
        let mock = MockHttpClient::new().with_response(
            "https://example.com",
            Ok(HttpResponse {
                status: 200,
                body: String::new(),
                headers,
            }),
        );

        let resp = mock.get("https://example.com").await.unwrap();
        assert_eq!(resp.headers.get("content-type").unwrap(), "text/html");
    }
}
