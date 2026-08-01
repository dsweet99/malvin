//! Transport-layer errors (not OpenRouter-prefixed).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("LLM unauthorized (401): {body}")]
    Unauthorized { body: String },
    #[error("OpenRouter billing/credit failure ({status}): {body}")]
    BillingFailure { status: u16, body: String },
    #[error("LLM rate limited (429): {body}")]
    RateLimited { body: String },
    #[error("LLM server error ({status}): {body}")]
    ServerError { status: u16, body: String },
    #[error("LLM request failed ({status}): {body}")]
    RequestFailed { status: u16, body: String },
    #[error("LLM context overflow: {body}")]
    ContextOverflow {
        body: String,
        message_count: usize,
    },
    #[error("LLM response missing assistant content")]
    MissingContent,
    #[error("{provider}: {detail}")]
    ProviderTransport { provider: String, detail: String },
    #[error("{provider}: {detail}")]
    ProviderError { provider: String, detail: String },
    #[error("HTTP transport error: {0}")]
    Network(String),
    #[error("Engine error: {0}")]
    Engine(String),
    #[error("JSON decode error: {0}")]
    Json(String),
    #[error("{0}")]
    Other(String),
}

impl TransportError {
    pub const FAIL_FAST_MARKER: &'static str = "MALVIN_MINI_MISSING_CONTENT_FAIL_FAST_V1";

    #[must_use]
    pub const fn is_billing_failure(&self) -> bool {
        matches!(self, Self::BillingFailure { .. })
    }

    #[must_use]
    pub const fn is_provider_error(&self) -> bool {
        matches!(self, Self::ProviderError { .. })
    }

    #[must_use]
    pub const fn is_transport_retryable(&self) -> bool {
        !self.is_billing_failure()
            && !self.is_context_overflow()
            && !self.is_provider_error()
            && !matches!(self, Self::MissingContent)
    }

    #[must_use]
    pub const fn is_context_overflow(&self) -> bool {
        matches!(self, Self::ContextOverflow { .. })
    }
}

/// True when a provider/HTTP body indicates the *input prompt* exceeded context budget.
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
pub fn is_prompt_too_long_error(err: &TransportError) -> bool {
    body_indicates_prompt_too_long(&err.to_string())
}

impl From<reqwest::Error> for TransportError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::TransportError;

    #[test]
    fn transport_error_billing_failure_is_not_transport_retryable() {
        assert!(TransportError::BillingFailure {
            status: 402,
            body: "no credits".into()
        }
        .is_billing_failure());
        assert!(!TransportError::BillingFailure {
            status: 403,
            body: "forbidden".into()
        }
        .is_transport_retryable());
    }

    #[test]
    fn transport_error_provider_error_is_not_transport_retryable() {
        let err = TransportError::ProviderError {
            provider: "Nvidia".into(),
            detail: "Conversation roles must alternate user/assistant/user/assistant/...".into(),
        };
        assert!(err.is_provider_error());
        assert!(!err.is_transport_retryable());
    }

    #[test]
    fn transport_error_transport_retryable_for_non_billing_failures() {
        assert!(TransportError::RateLimited {
            body: "slow".into()
        }
        .is_transport_retryable());
        assert!(TransportError::ServerError {
            status: 503,
            body: "down".into()
        }
        .is_transport_retryable());
        assert!(TransportError::Unauthorized {
            body: "bad".into()
        }
        .is_transport_retryable());
        assert!(TransportError::RequestFailed {
            status: 418,
            body: "teapot".into()
        }
        .is_transport_retryable());
        assert!(!TransportError::MissingContent.is_transport_retryable());
        let json = TransportError::Json("bad".into());
        assert!(json.is_transport_retryable());
        assert!(TransportError::ProviderTransport {
            provider: "Nvidia".into(),
            detail: "ResourceExhausted".into(),
        }
        .is_transport_retryable());
    }

    #[test]
    fn transport_error_context_overflow_is_not_transport_retryable() {
        assert!(!TransportError::ContextOverflow {
            body: "too long".into(),
            message_count: 1,
        }
        .is_transport_retryable());
    }

    #[test]
    fn is_prompt_too_long_error_matches_request_failed_body() {
        let err = TransportError::RequestFailed {
            status: 400,
            body: r#"{"error":"prompt is too long"}"#.into(),
        };
        assert!(super::is_prompt_too_long_error(&err));
        let live = TransportError::ProviderError {
            provider: "Provider".into(),
            detail: "Prompt tokens limit exceeded: 21287 > 13840".into(),
        };
        assert!(super::is_prompt_too_long_error(&live));
        assert!(super::body_indicates_prompt_too_long(
            "Prompt tokens limit exceeded: 21287 > 13840"
        ));
        assert!(!super::is_prompt_too_long_error(&TransportError::RateLimited {
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
