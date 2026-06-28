use malvin_mini::{CompletionResponse, HttpExchangeMeta, OpenRouterError};

pub(super) fn mock_json_error() -> OpenRouterError {
    OpenRouterError::Json(
        serde_json::from_str::<serde_json::Value>("not json").unwrap_err(),
    )
}

pub(super) fn mock_http_meta(status: Option<u16>, body: Option<&str>) -> HttpExchangeMeta {
    HttpExchangeMeta {
        status,
        body: body.map(str::to_string),
    }
}

pub(super) fn mock_ok_pair(response: &CompletionResponse) -> (Result<CompletionResponse, OpenRouterError>, HttpExchangeMeta) {
    (Ok(response.clone()), mock_http_meta(Some(200), None))
}

pub(super) fn mock_rate_limited_pair() -> (Result<CompletionResponse, OpenRouterError>, HttpExchangeMeta) {
    (
        Err(OpenRouterError::RateLimited { body: "slow".into() }),
        mock_http_meta(Some(429), Some("slow")),
    )
}

pub(super) fn mock_context_overflow_pair(
    messages_len: usize,
) -> (Result<CompletionResponse, OpenRouterError>, HttpExchangeMeta) {
    (
        Err(OpenRouterError::ContextOverflow {
            body: "prompt is too long".into(),
            message_count: messages_len,
        }),
        mock_http_meta(Some(400), Some("prompt is too long")),
    )
}

pub(super) fn mock_request_failed_pair(
    status: u16,
    body: &str,
) -> (Result<CompletionResponse, OpenRouterError>, HttpExchangeMeta) {
    (
        Err(OpenRouterError::RequestFailed {
            status,
            body: body.to_string(),
        }),
        mock_http_meta(Some(status), Some(body)),
    )
}

pub(super) fn mock_billing_failure_pair(
    status: u16,
    body: &str,
) -> (Result<CompletionResponse, OpenRouterError>, HttpExchangeMeta) {
    (
        Err(OpenRouterError::BillingFailure {
            status,
            body: body.to_string(),
        }),
        mock_http_meta(Some(status), Some(body)),
    )
}

pub(super) fn mock_provider_transport_pair() -> (Result<CompletionResponse, OpenRouterError>, HttpExchangeMeta) {
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
        Err(OpenRouterError::ProviderTransport {
            provider: "Nvidia".into(),
            detail: "ResourceExhausted".into(),
        }),
        mock_http_meta(Some(200), Some(body)),
    )
}

pub(super) fn mock_json_transport_pair() -> (Result<CompletionResponse, OpenRouterError>, HttpExchangeMeta) {
    (
        Err(mock_json_error()),
        mock_http_meta(Some(200), Some("not json")),
    )
}
