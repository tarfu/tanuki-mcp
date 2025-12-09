//! GitLab API client
//!
//! Provides a typed HTTP client for interacting with the GitLab REST API.

use crate::auth::BoxedAuthProvider;
use crate::config::GitLabConfig;
use crate::error::{GitLabError, GitLabResult};
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};

/// GitLab API client
pub struct GitLabClient {
    http: Client,
    base_url: String,
    auth: Arc<RwLock<BoxedAuthProvider>>,
    max_retries: u32,
}

impl GitLabClient {
    /// Create a new GitLab client from configuration
    pub fn new(config: &GitLabConfig, auth: BoxedAuthProvider) -> GitLabResult<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .danger_accept_invalid_certs(!config.verify_ssl)
            .user_agent(format!("tanuki-mcp/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(GitLabError::Request)?;

        Ok(Self {
            http,
            base_url: config.api_url(),
            auth: Arc::new(RwLock::new(auth)),
            max_retries: config.max_retries,
        })
    }

    /// Build a URL for an API endpoint
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Add authentication to a request
    async fn authenticate(&self, request: RequestBuilder) -> GitLabResult<RequestBuilder> {
        let auth = self.auth.read().await;
        let header = auth.get_auth_header().await.map_err(|e| GitLabError::Api {
            status: 401,
            message: e.to_string(),
        })?;

        Ok(request.header(header.header_name(), header.header_value()))
    }

    /// Execute a request with retries
    async fn execute(&self, request: RequestBuilder) -> GitLabResult<Response> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                // Exponential backoff
                let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
                debug!("Retrying request (attempt {})", attempt + 1);
            }

            // Clone the request for retry
            let req = request
                .try_clone()
                .ok_or_else(|| GitLabError::InvalidResponse("Cannot clone request".to_string()))?;

            match req.send().await {
                Ok(response) => {
                    return self.handle_response(response).await;
                }
                Err(e) => {
                    warn!("Request failed: {}", e);
                    last_error = Some(GitLabError::Request(e));

                    // Only retry on connection/timeout errors
                    if !is_retryable(last_error.as_ref().unwrap()) {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| GitLabError::InvalidResponse("Unknown error".to_string())))
    }

    /// Handle API response
    async fn handle_response(&self, response: Response) -> GitLabResult<Response> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        // Extract error details from response body
        let body = response.text().await.unwrap_or_default();

        // Check for rate limiting
        if status == StatusCode::TOO_MANY_REQUESTS {
            // Try to parse retry-after from response
            let retry_after = 60; // Default
            return Err(GitLabError::RateLimited { retry_after });
        }

        Err(GitLabError::from_response(status.as_u16(), &body))
    }

    /// Make a GET request
    #[instrument(skip(self), fields(endpoint = %endpoint))]
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> GitLabResult<T> {
        let url = self.url(endpoint);
        let request = self.http.get(&url);
        let request = self.authenticate(request).await?;

        let response = self.execute(request).await?;
        let data = response.json().await.map_err(|e| {
            GitLabError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(data)
    }

    /// Make a GET request returning raw JSON value
    pub async fn get_json(&self, endpoint: &str) -> GitLabResult<serde_json::Value> {
        self.get(endpoint).await
    }

    /// Make a GET request returning raw text (not JSON)
    #[instrument(skip(self), fields(endpoint = %endpoint))]
    pub async fn get_text(&self, endpoint: &str) -> GitLabResult<String> {
        let url = self.url(endpoint);
        let request = self.http.get(&url);
        let request = self.authenticate(request).await?;

        let response = self.execute(request).await?;
        let text = response.text().await.map_err(|e| {
            GitLabError::InvalidResponse(format!("Failed to read response text: {}", e))
        })?;

        Ok(text)
    }

    /// Make a POST request
    #[instrument(skip(self, body), fields(endpoint = %endpoint))]
    pub async fn post<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> GitLabResult<T> {
        let url = self.url(endpoint);
        let request = self.http.post(&url).json(body);
        let request = self.authenticate(request).await?;

        let response = self.execute(request).await?;
        let data = response.json().await.map_err(|e| {
            GitLabError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(data)
    }

    /// Make a POST request returning raw JSON value
    pub async fn post_json<B: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> GitLabResult<serde_json::Value> {
        self.post(endpoint, body).await
    }

    /// Make a POST request that expects no content in response (HTTP 204)
    pub async fn post_no_content<B: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> GitLabResult<()> {
        let url = self.url(endpoint);
        let request = self.http.post(&url).json(body);
        let request = self.authenticate(request).await?;

        self.execute(request).await?;
        Ok(())
    }

    /// Make a PUT request
    #[instrument(skip(self, body), fields(endpoint = %endpoint))]
    pub async fn put<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> GitLabResult<T> {
        let url = self.url(endpoint);
        let request = self.http.put(&url).json(body);
        let request = self.authenticate(request).await?;

        let response = self.execute(request).await?;
        let data = response.json().await.map_err(|e| {
            GitLabError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(data)
    }

    /// Make a PUT request returning raw JSON value
    pub async fn put_json<B: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> GitLabResult<serde_json::Value> {
        self.put(endpoint, body).await
    }

    /// Make a PUT request that expects no content in response (HTTP 204)
    pub async fn put_no_content<B: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> GitLabResult<()> {
        let url = self.url(endpoint);
        let request = self.http.put(&url).json(body);
        let request = self.authenticate(request).await?;

        self.execute(request).await?;
        Ok(())
    }

    /// Make a DELETE request
    #[instrument(skip(self), fields(endpoint = %endpoint))]
    pub async fn delete(&self, endpoint: &str) -> GitLabResult<()> {
        let url = self.url(endpoint);
        let request = self.http.delete(&url);
        let request = self.authenticate(request).await?;

        self.execute(request).await?;
        Ok(())
    }

    /// Make a DELETE request with a body
    pub async fn delete_with_body<B: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> GitLabResult<()> {
        let url = self.url(endpoint);
        let request = self.http.delete(&url).json(body);
        let request = self.authenticate(request).await?;

        self.execute(request).await?;
        Ok(())
    }

    /// Make a request with custom method
    pub async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
    ) -> GitLabResult<T> {
        let url = self.url(endpoint);
        let request = self.http.request(method, &url);
        let request = self.authenticate(request).await?;

        let response = self.execute(request).await?;
        let data = response.json().await.map_err(|e| {
            GitLabError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(data)
    }

    /// Make a request with custom method and body
    pub async fn request_with_body<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        method: Method,
        endpoint: &str,
        body: &B,
    ) -> GitLabResult<T> {
        let url = self.url(endpoint);
        let request = self.http.request(method, &url).json(body);
        let request = self.authenticate(request).await?;

        let response = self.execute(request).await?;
        let data = response.json().await.map_err(|e| {
            GitLabError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(data)
    }

    /// URL-encode a project path for use in API endpoints
    pub fn encode_project(project: &str) -> String {
        urlencoding::encode(project).to_string()
    }
}

/// Check if an error is retryable
fn is_retryable(error: &GitLabError) -> bool {
    match error {
        GitLabError::Request(e) => e.is_timeout() || e.is_connect(),
        GitLabError::RateLimited { .. } => true,
        GitLabError::Api { status, .. } => *status >= 500,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_project() {
        assert_eq!(
            GitLabClient::encode_project("group/project"),
            "group%2Fproject"
        );
        assert_eq!(
            GitLabClient::encode_project("group/subgroup/project"),
            "group%2Fsubgroup%2Fproject"
        );
    }

    #[test]
    fn test_is_retryable() {
        assert!(is_retryable(&GitLabError::RateLimited { retry_after: 60 }));
        assert!(is_retryable(&GitLabError::Api {
            status: 500,
            message: "Internal error".to_string()
        }));
        assert!(is_retryable(&GitLabError::Api {
            status: 503,
            message: "Service unavailable".to_string()
        }));
        assert!(!is_retryable(&GitLabError::Api {
            status: 400,
            message: "Bad request".to_string()
        }));
        assert!(!is_retryable(&GitLabError::Unauthorized));
    }
}
