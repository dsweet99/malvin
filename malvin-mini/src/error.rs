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
    #[error("{provider}: {detail}")]
    ProviderTransport { provider: String, detail: String },
    #[error("{provider}: {detail}")]
    ProviderError { provider: String, detail: String },
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("JSON decode error: {0}")]
    Json(#[from] serde_json::Error),
}

impl OpenRouterError {
    pub const FAIL_FAST_MARKER: &'static str = "MALVIN_MINI_MISSING_CONTENT_FAIL_FAST_V1";

    #[must_use]
    pub const fn is_billing_failure(&self) -> bool {
        matches!(self, Self::BillingFailure { .. })
    }

    #[must_use]
    pub const fn is_provider_error(&self) -> bool {
        matches!(self, Self::ProviderError { .. })
    }

    /// True for every error that should consume the mini transport retry budget.
    ///
    /// `MissingContent` is handled inside `complete` with local shape retries; after that
    /// budget is exhausted, outer ACP/gate transport retries only burn wall time.
    #[must_use]
    pub const fn is_transport_retryable(&self) -> bool {
        !self.is_billing_failure()
            && !self.is_context_overflow()
            && !self.is_provider_error()
            && !matches!(self, Self::MissingContent) // MALVIN_MINI_MISSING_CONTENT_FAIL_FAST_V1
    }

    #[must_use]
    pub const fn is_context_overflow(&self) -> bool {
        matches!(self, Self::ContextOverflow { .. })
    }
}

/// True when a provider/HTTP body indicates the *input prompt* exceeded context budget
/// (distinct from completion `max_tokens` affordability).
#[must_use]
pub fn body_indicates_prompt_too_long(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    lower.contains("prompt is too long")
        || lower.contains("prompt tokens limit exceeded")
        || (lower.contains("prompt token") && lower.contains("limit exceeded"))
        || (lower.contains("context length") && lower.contains("exceed"))
        || (lower.contains("maximum context length") && lower.contains("exceed"))
}

#[must_use]
pub fn is_prompt_too_long_error(err: &OpenRouterError) -> bool {
    body_indicates_prompt_too_long(&err.to_string())
}

#[cfg(test)]
mod tests {
    use super::OpenRouterError;

    #[test]
    fn openrouter_error_billing_failure_is_not_transport_retryable() {
        assert!(OpenRouterError::BillingFailure {
            status: 402,
            body: "no credits".into()
        }
        .is_billing_failure());
        assert!(!OpenRouterError::BillingFailure {
            status: 403,
            body: "forbidden".into()
        }
        .is_transport_retryable());
    }

    #[test]
    fn openrouter_error_provider_error_is_not_transport_retryable() {
        let err = OpenRouterError::ProviderError {
            provider: "Nvidia".into(),
            detail: "Conversation roles must alternate user/assistant/user/assistant/...".into(),
        };
        assert!(err.is_provider_error());
        assert!(!err.is_transport_retryable());
        assert_eq!(
            err.to_string(),
            "Nvidia: Conversation roles must alternate user/assistant/user/assistant/..."
        );
    }

    #[test]
    fn openrouter_error_transport_retryable_for_non_billing_failures() {
        assert!(OpenRouterError::RateLimited {
            body: "slow".into()
        }
        .is_transport_retryable());
        assert!(OpenRouterError::ServerError {
            status: 503,
            body: "down".into()
        }
        .is_transport_retryable());
        assert!(OpenRouterError::Unauthorized {
            body: "bad".into()
        }
        .is_transport_retryable());
        assert!(OpenRouterError::RequestFailed {
            status: 418,
            body: "teapot".into()
        }
        .is_transport_retryable());
        assert!(!OpenRouterError::MissingContent.is_transport_retryable());
        let json = OpenRouterError::Json(serde_json::from_str::<serde_json::Value>("not json").unwrap_err());
        assert!(json.is_transport_retryable());
        assert!(OpenRouterError::ProviderTransport {
            provider: "Nvidia".into(),
            detail: "ResourceExhausted".into(),
        }
        .is_transport_retryable());
    }

    #[test]
    fn openrouter_error_context_overflow_is_not_transport_retryable() {
        assert!(!OpenRouterError::ContextOverflow {
            body: "too long".into(),
            message_count: 1,
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
        let live = OpenRouterError::ProviderError {
            provider: "Provider".into(),
            detail: "Prompt tokens limit exceeded: 21287 > 13840".into(),
        };
        assert!(super::is_prompt_too_long_error(&live));
        assert!(super::body_indicates_prompt_too_long(
            "Prompt tokens limit exceeded: 21287 > 13840"
        ));
        assert!(!super::is_prompt_too_long_error(&OpenRouterError::RateLimited {
            body: "slow".into()
        }));
    }

    #[test]
    fn kiss_cov_is_prompt_too_long_error() {
        let _ = (
            super::is_prompt_too_long_error,
            super::body_indicates_prompt_too_long,
        );
    }
}
