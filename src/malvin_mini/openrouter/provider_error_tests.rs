//! Unit tests for provider error mapping.

use super::provider_error::{
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
fn provider_fatal_maps_credit_affordability_to_billing_failure() {
    let body = r#"{
        "error": {
            "message": "This request requires more credits, or fewer max_tokens. You requested up to 65536 tokens, but can only afford 55314.",
            "code": 402,
            "metadata": {
                "provider_name": "Provider"
            }
        }
    }"#;
    let err = provider_fatal_from_body(body).expect("billing");
    assert!(err.is_billing_failure());
    assert!(!err.is_transport_retryable());
    assert!(provider_transport_from_body(body).is_none());
}

#[test]
fn provider_fatal_maps_insufficient_credits_to_billing_failure() {
    let body = r#"{
        "error": {
            "message": "Insufficient credits. Add more using https://openrouter.ai/settings/credits",
            "code": 402,
            "metadata": {
                "provider_name": "Provider"
            }
        }
    }"#;
    let err = provider_fatal_from_body(body).expect("billing");
    assert!(err.is_billing_failure());
    assert!(!err.is_transport_retryable());
    assert!(
        err.to_string()
            .contains("OpenRouter billing/credit failure"),
        "{}",
        err
    );
    assert!(provider_transport_from_body(body).is_none());
}

#[test]
fn extract_raw_message_reads_object_message_field() {
    let raw = serde_json::json!({"message": "ResourceExhausted"});
    assert_eq!(extract_raw_message(&raw), "ResourceExhausted");
}
