use crate::error::OpenRouterError;

/// When OpenRouter returns a provider-side capacity error (e.g. Nvidia
/// `ResourceExhausted`), surface `{provider}: {detail}` and retry as transport.
pub(crate) fn provider_transport_from_body(body: &str) -> Option<OpenRouterError> {
    let (provider, error_type, top_message, raw_detail) = parse_provider_error_envelope(body)?;
    let detail = select_provider_detail(&top_message, &raw_detail);
    if is_provider_transport_retryable(&error_type, &detail, &raw_detail) {
        Some(OpenRouterError::ProviderTransport { provider, detail })
    } else {
        None
    }
}

/// When OpenRouter returns HTTP 200 with a non-retryable provider error envelope,
/// surface `{provider}: {detail}` without retrying.
pub(crate) fn provider_fatal_from_body(body: &str) -> Option<OpenRouterError> {
    let (provider, error_type, top_message, raw_detail) = parse_provider_error_envelope(body)?;
    if is_provider_transport_retryable(&error_type, &top_message, &raw_detail) {
        return None;
    }
    let detail = select_provider_detail(&top_message, &raw_detail);
    Some(OpenRouterError::ProviderError { provider, detail })
}

fn parse_provider_error_envelope(body: &str) -> Option<(String, String, String, String)> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let error = value.get("error")?;
    let metadata = error.get("metadata");
    Some((
        metadata_string(metadata, "provider_name").unwrap_or_else(|| "Provider".to_string()),
        metadata_string(metadata, "error_type").unwrap_or_default(),
        error
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string(),
        metadata
            .and_then(|m| m.get("raw"))
            .map(extract_raw_message)
            .unwrap_or_default(),
    ))
}

fn metadata_string(metadata: Option<&serde_json::Value>, key: &str) -> Option<String> {
    metadata
        .and_then(|m| m.get(key))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn select_provider_detail(top_message: &str, raw_detail: &str) -> String {
    if raw_detail.is_empty() {
        top_message.to_string()
    } else if top_message.eq_ignore_ascii_case("Provider returned error")
        || raw_detail.eq_ignore_ascii_case(top_message)
        || raw_detail.contains(top_message)
    {
        raw_detail.to_string()
    } else {
        format!("{top_message}: {raw_detail}")
    }
}

fn extract_raw_message(raw: &serde_json::Value) -> String {
    match raw {
        serde_json::Value::String(text) => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                extract_raw_message(&parsed)
            } else {
                text.clone()
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(message) = map.get("message").and_then(serde_json::Value::as_str) {
                return message.to_string();
            }
            if let Some(error) = map.get("error") {
                return extract_raw_message(error);
            }
            String::new()
        }
        _ => String::new(),
    }
}

fn is_provider_transport_retryable(error_type: &str, detail: &str, raw_detail: &str) -> bool {
    let error_type = error_type.to_ascii_lowercase();
    if error_type.contains("overloaded") || error_type.contains("unavailable") {
        return true;
    }
    let combined = format!("{detail} {raw_detail}").to_ascii_lowercase();
    combined.contains("resourceexhausted")
        || combined.contains("resource exhausted")
        || combined.contains("provider is overloaded")
}

#[cfg(test)]
mod tests {
    use super::{
        extract_raw_message, provider_fatal_from_body, provider_transport_from_body,
        select_provider_detail,
    };

    #[test]
    fn provider_transport_from_http_200_nvidia_resource_exhausted() {
        let body = r#"{
            "error": {
                "message": "Provider returned error",
                "code": 503,
                "metadata": {
                    "provider_name": "Nvidia",
                    "raw": "{\"error\":{\"message\":\"ResourceExhausted\",\"type\":\"invalid_request_error\"}}",
                    "error_type": "provider_overloaded"
                }
            }
        }"#;
        let err = provider_transport_from_body(body).expect("provider transport");
        assert!(err.is_transport_retryable());
        assert_eq!(err.to_string(), "Nvidia: ResourceExhausted");
    }

    #[test]
    fn provider_transport_from_non_200_provider_overloaded() {
        let body = r#"{
            "error": {
                "message": "Provider returned error",
                "code": 503,
                "metadata": {
                    "provider_name": "Nvidia",
                    "raw": "ResourceExhausted",
                    "error_type": "provider_unavailable"
                }
            }
        }"#;
        let err = provider_transport_from_body(body).expect("provider transport");
        assert_eq!(err.to_string(), "Nvidia: ResourceExhausted");
    }

    #[test]
    fn provider_fatal_from_http_200_invalid_request() {
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
        let err = provider_fatal_from_body(body).expect("provider fatal");
        assert!(!err.is_transport_retryable());
        assert_eq!(
            err.to_string(),
            "Nvidia: Conversation roles must alternate user/assistant/user/assistant/..."
        );
    }

    #[test]
    fn provider_transport_skips_non_retryable_provider_errors() {
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
        assert!(provider_transport_from_body(body).is_none());
    }

    #[test]
    fn extract_raw_message_parses_nested_json_string() {
        let raw: serde_json::Value = serde_json::from_str(
            r#""{\"error\":{\"message\":\"ResourceExhausted\"}}""#,
        )
        .expect("json");
        assert_eq!(extract_raw_message(&raw), "ResourceExhausted");
    }

    #[test]
    fn select_provider_detail_prefers_raw_over_generic_top_message() {
        assert_eq!(
            select_provider_detail("Provider returned error", "ResourceExhausted"),
            "ResourceExhausted"
        );
    }

    #[test]
    fn provider_transport_returns_none_for_completion_body() {
        let body = r#"{"choices":[{"message":{"content":"ok"}}]}"#;
        assert!(provider_transport_from_body(body).is_none());
    }

    #[test]
    fn provider_transport_returns_none_for_invalid_json() {
        assert!(provider_transport_from_body("not json").is_none());
    }

    #[test]
    fn provider_transport_uses_default_provider_name() {
        let body = r#"{
            "error": {
                "message": "ResourceExhausted",
                "metadata": {
                    "error_type": "provider_overloaded"
                }
            }
        }"#;
        let err = provider_transport_from_body(body).expect("provider transport");
        assert_eq!(err.to_string(), "Provider: ResourceExhausted");
    }

    #[test]
    fn select_provider_detail_joins_distinct_top_and_raw_messages() {
        assert_eq!(
            select_provider_detail("upstream busy", "retry later"),
            "upstream busy: retry later"
        );
        assert_eq!(select_provider_detail("busy", ""), "busy");
    }

    #[test]
    fn extract_raw_message_reads_object_message_field() {
        let raw = serde_json::json!({"message": "ResourceExhausted"});
        assert_eq!(extract_raw_message(&raw), "ResourceExhausted");
    }
}
