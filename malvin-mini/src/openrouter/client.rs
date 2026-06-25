use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

use crate::config::OpenRouterConfig;
use crate::error::OpenRouterError;

pub struct OpenRouterClient {
    http: reqwest::Client,
    config: OpenRouterConfig,
}

impl OpenRouterClient {
    /// # Errors
    ///
    /// Returns [`OpenRouterError`] when the HTTP client cannot be built.
    pub fn new(config: OpenRouterConfig) -> Result<Self, OpenRouterError> {
        let http = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .build()?;
        Ok(Self { http, config })
    }

    #[must_use]
    pub const fn config(&self) -> &OpenRouterConfig {
        &self.config
    }

    #[must_use]
    pub(super) fn http(&self) -> &reqwest::Client {
        &self.http
    }
}

pub(super) fn build_request_headers(config: &OpenRouterConfig) -> Result<HeaderMap, OpenRouterError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", config.api_key)).map_err(|e| {
            OpenRouterError::RequestFailed {
                status: 0,
                body: format!("invalid authorization header: {e}"),
            }
        })?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Some(ref referer) = config.http_referer {
        headers.insert(
            "HTTP-Referer",
            HeaderValue::from_str(referer).map_err(|e| OpenRouterError::RequestFailed {
                status: 0,
                body: format!("invalid HTTP-Referer header: {e}"),
            })?,
        );
    }
    headers.insert("X-OpenRouter-Title", HeaderValue::from_static("malvin"));
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{build_request_headers, OpenRouterClient};
    use crate::config::OpenRouterConfig;

    #[test]
    fn build_request_headers_includes_auth_and_referer() {
        let config = OpenRouterConfig {
            model: "m".into(),
            api_key: "sk-test".into(),
            http_referer: Some("https://malvin.test".into()),
            request_timeout: Duration::from_secs(30),
            base_url: "https://openrouter.ai/api/v1".into(),
        };
        let headers = build_request_headers(&config).expect("headers");
        assert_eq!(
            headers.get("authorization").and_then(|v| v.to_str().ok()),
            Some("Bearer sk-test")
        );
        assert_eq!(
            headers.get("http-referer").and_then(|v| v.to_str().ok()),
            Some("https://malvin.test")
        );
    }

    #[test]
    fn openrouter_client_new_exposes_config_and_http() {
        let config = OpenRouterConfig {
            model: "m".into(),
            api_key: "sk-test".into(),
            http_referer: None,
            request_timeout: Duration::from_secs(30),
            base_url: "https://openrouter.ai/api/v1".into(),
        };
        let client = OpenRouterClient::new(config).expect("client");
        assert_eq!(client.config().model, "m");
        assert!(client.http().get("https://example.com").build().is_ok());
    }
}
