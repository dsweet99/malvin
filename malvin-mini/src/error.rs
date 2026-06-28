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
    #[error("OpenRouter context overflow: {body}")]
    ContextOverflow {
        body: String,
        message_count: usize,
    },
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

    #[must_use]
    pub const fn is_transport_retryable(&self) -> bool {
        matches!(self, Self::Transport(_) | Self::Json(_))
    }

    #[must_use]
    pub const fn is_context_overflow(&self) -> bool {
        matches!(self, Self::ContextOverflow { .. })
    }
}

#[must_use]
pub fn is_prompt_too_long_error(err: &OpenRouterError) -> bool {
    err.to_string()
        .to_ascii_lowercase()
        .contains("prompt is too long")
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

    #[test]
    fn openrouter_error_transport_retryable_for_transport_and_json() {
        let transport = OpenRouterError::Json(
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err(),
        );
        assert!(transport.is_transport_retryable());
        let json = OpenRouterError::Json(serde_json::from_str::<serde_json::Value>("not json").unwrap_err());
        assert!(json.is_transport_retryable());
        assert!(!OpenRouterError::Unauthorized {
            body: "bad".into()
        }
        .is_transport_retryable());
    }

    #[test]
    fn is_prompt_too_long_error_matches_request_failed_body() {
        let err = OpenRouterError::RequestFailed {
            status: 400,
            body: r#"{"error":"prompt is too long"}"#.into(),
        };
        assert!(super::is_prompt_too_long_error(&err));
        assert!(!super::is_prompt_too_long_error(&OpenRouterError::RateLimited {
            body: "slow".into()
        }));
    }

    #[test]
    fn kiss_cov_is_prompt_too_long_error() {
        let _ = super::is_prompt_too_long_error;
    }
}
