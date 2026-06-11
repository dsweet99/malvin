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

    #[test]
    fn openrouter_error_retryable_for_rate_limit_and_server_error() {
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
