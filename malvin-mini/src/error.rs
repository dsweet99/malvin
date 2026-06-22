use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenRouterError {
    #[error("OpenRouter unauthorized (401): {body}")]
    Unauthorized { body: String },
    #[error("OpenRouter billing/credit failure ({status}): {body}")]
    BillingFailure { status: u16, body: String },
    #[error("OpenRouter rate limited (429): {body}")]
    RateLimited { body: String },
    #[error("OpenRouter server error ({status}): {body}")]
    ServerError { status: u16, body: String },
    #[error("OpenRouter request failed ({status}): {body}")]
    RequestFailed { status: u16, body: String },
    #[error("OpenRouter response missing assistant content")]
    MissingContent,
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("JSON decode error: {0}")]
    Json(#[from] serde_json::Error),
}

impl OpenRouterError {
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::ServerError { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::OpenRouterError;

    fn fake_reqwest_error() -> reqwest::Error {
        reqwest::Client::new()
            .get("http:///")
            .build()
            .expect_err("invalid url")
    }

    #[test]
    fn openrouter_error_retryable_for_rate_limit_and_server_error() {
        let _variants: [OpenRouterError; 8] = [
            OpenRouterError::Unauthorized { body: String::new() },
            OpenRouterError::BillingFailure {
                status: 402,
                body: String::new(),
            },
            OpenRouterError::RateLimited { body: String::new() },
            OpenRouterError::ServerError {
                status: 500,
                body: String::new(),
            },
            OpenRouterError::RequestFailed {
                status: 400,
                body: String::new(),
            },
            OpenRouterError::MissingContent,
            OpenRouterError::Transport(fake_reqwest_error()),
            OpenRouterError::Json(serde_json::from_str::<()>("").unwrap_err()),
        ];
        let _ = _variants;
        assert!(OpenRouterError::RateLimited {
            body: "slow".into()
        }
        .is_retryable());
        assert!(OpenRouterError::ServerError {
            status: 503,
            body: "down".into()
        }
        .is_retryable());
        assert!(!OpenRouterError::Unauthorized {
            body: "bad".into()
        }
        .is_retryable());
        assert!(!OpenRouterError::RequestFailed {
            status: 418,
            body: "teapot".into()
        }
        .is_retryable());
    }
}
