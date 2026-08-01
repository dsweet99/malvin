use crate::openrouter_transport::{CompletionResponse, HttpExchangeMeta, TransportError};

pub(super) fn mock_json_error() -> TransportError {
    TransportError::Json(
        serde_json::from_str::<serde_json::Value>("not json")
            .unwrap_err()
            .to_string(),
    )
}

pub(super) fn mock_http_meta(status: Option<u16>, body: Option<&str>) -> HttpExchangeMeta {
    HttpExchangeMeta {
        status,
        body: body.map(str::to_string),
    }
}

pub(super) fn mock_ok_pair(response: &CompletionResponse) -> (Result<CompletionResponse, TransportError>, HttpExchangeMeta) {
    (Ok(response.clone()), mock_http_meta(Some(200), None))
}

pub(super) fn mock_rate_limited_pair() -> (Result<CompletionResponse, TransportError>, HttpExchangeMeta) {
    (
        Err(TransportError::RateLimited { body: "slow".into() }),
        mock_http_meta(Some(429), Some("slow")),
    )
}

pub(super) fn mock_context_overflow_pair(
    messages_len: usize,
) -> (Result<CompletionResponse, TransportError>, HttpExchangeMeta) {
    (
        Err(TransportError::ContextOverflow {
            body: "prompt is too long".into(),
            message_count: messages_len,
        }),
        mock_http_meta(Some(400), Some("prompt is too long")),
    )
}

pub(super) fn mock_request_failed_pair(
    status: u16,
    body: &str,
) -> (Result<CompletionResponse, TransportError>, HttpExchangeMeta) {
    (
        Err(TransportError::RequestFailed {
            status,
            body: body.to_string(),
        }),
        mock_http_meta(Some(status), Some(body)),
    )
}

pub(super) fn mock_billing_failure_pair(
    status: u16,
    body: &str,
) -> (Result<CompletionResponse, TransportError>, HttpExchangeMeta) {
    (
        Err(TransportError::BillingFailure {
            status,
            body: body.to_string(),
        }),
        mock_http_meta(Some(status), Some(body)),
    )
}

pub(super) fn mock_provider_fatal_pair() -> (Result<CompletionResponse, TransportError>, HttpExchangeMeta) {
    let body = r#"{
        "error": {
            "message": "Provider returned error",
            "code": 400,
            "metadata": {
                "provider_name": "Nvidia",
                "raw": "{\"error\":{\"message\":\"Conversation roles must alternate user/assistant/user/assistant/...\"}}",
                "error_type": "invalid_request"
            }
        }
    }"#;
    (
        Err(TransportError::ProviderError {
            provider: "Nvidia".into(),
            detail: "Conversation roles must alternate user/assistant/user/assistant/...".into(),
        }),
        mock_http_meta(Some(200), Some(body)),
    )
}

pub(super) fn mock_provider_transport_pair() -> (Result<CompletionResponse, TransportError>, HttpExchangeMeta) {
    let body = r#"{
        "error": {
            "message": "Provider returned error",
            "code": 503,
            "metadata": {
                "provider_name": "Nvidia",
                "raw": "ResourceExhausted",
                "error_type": "provider_overloaded"
            }
        }
    }"#;
    (
        Err(TransportError::ProviderTransport {
            provider: "Nvidia".into(),
            detail: "ResourceExhausted".into(),
        }),
        mock_http_meta(Some(200), Some(body)),
    )
}

pub(super) fn mock_json_transport_pair() -> (Result<CompletionResponse, TransportError>, HttpExchangeMeta) {
    (
        Err(mock_json_error()),
        mock_http_meta(Some(200), Some("not json")),
    )
}
