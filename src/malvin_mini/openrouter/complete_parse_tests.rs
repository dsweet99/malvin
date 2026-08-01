use super::complete_parse::{map_http_status, outcome_from_http_body, parse_completion_body};
use crate::malvin_mini::error::OpenRouterError;

#[test]
fn map_http_status_maps_known_codes() {
    assert!(map_http_status(200, "").is_ok());
    assert!(matches!(
        map_http_status(401, "bad").unwrap_err(),
        OpenRouterError::Unauthorized { .. }
    ));
    assert!(matches!(
        map_http_status(429, "slow").unwrap_err(),
        OpenRouterError::RateLimited { .. }
    ));
    assert!(matches!(
        map_http_status(500, "boom").unwrap_err(),
        OpenRouterError::ServerError { .. }
    ));
    assert!(matches!(
        map_http_status(418, "teapot").unwrap_err(),
        OpenRouterError::RequestFailed { status: 418, .. }
    ));
}

#[test]
fn parse_completion_body_extracts_content_and_usage() {
    let body = r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":3}}"#;
    let resp = parse_completion_body(body).expect("parse");
    assert_eq!(resp.content, "ok");
    assert_eq!(resp.usage.and_then(|u| u.total_tokens), Some(3));
    let err = parse_completion_body(r#"{"choices":[{"message":{}}]}"#).expect_err("missing");
    assert!(matches!(err, OpenRouterError::MissingContent));
}

#[test]
fn parse_completion_body_accepts_content_parts_array() {
    let body = r#"{"choices":[{"message":{"content":[{"type":"text","text":"hello"}]}}]}"#;
    let resp = parse_completion_body(body).expect("parse parts");
    assert_eq!(resp.content, "hello");
}

#[test]
fn parse_completion_body_joins_multiple_content_parts() {
    let body = r#"{"choices":[{"message":{"content":[
            {"type":"text","text":"line1"},
            {"type":"text","text":"line2"}
        ]}}]}"#;
    let resp = parse_completion_body(body).expect("parse parts");
    assert_eq!(resp.content, "line1\nline2");
}

#[test]
fn parse_completion_body_prefers_non_empty_text_over_reasoning() {
    let body = r#"{"choices":[{"message":{"content":"answer","reasoning":"think"}}]}"#;
    let resp = parse_completion_body(body).expect("parse text");
    assert_eq!(resp.content, "answer");
}

#[test]
fn outcome_from_http_body_maps_non_retryable_provider_error() {
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
    let meta = outcome_from_http_body(200, body.into(), 1);
    let err = meta.result.expect_err("provider error");
    assert!(err.is_provider_error());
    assert!(!err.is_transport_retryable());
    assert_eq!(
        err.to_string(),
        "Nvidia: Conversation roles must alternate user/assistant/user/assistant/..."
    );
}

#[test]
fn outcome_from_http_body_maps_prompt_token_limit_to_context_overflow() {
    let body = r#"{"error":{"message":"Provider returned error","metadata":{"provider_name":"Provider","raw":"Prompt tokens limit exceeded: 21287 > 13840"}}}"#;
    let meta = outcome_from_http_body(400, body.into(), 4);
    let err = meta.result.expect_err("overflow");
    assert!(err.is_context_overflow());
    assert!(!err.is_transport_retryable());
}

#[test]
fn outcome_from_http_body_maps_402_insufficient_credits_to_billing() {
    let body =
        r#"{"error":{"message":"Insufficient credits. Add more using https://openrouter.ai/settings/credits","code":402}}"#;
    let meta = outcome_from_http_body(402, body.into(), 1);
    let err = meta.result.expect_err("billing");
    assert!(err.is_billing_failure());
    assert!(!err.is_transport_retryable());
    assert!(
        err.to_string()
            .contains("OpenRouter billing/credit failure"),
        "{}",
        err
    );
}

#[test]
fn parse_completion_body_does_not_promote_reasoning_to_content() {
    let body = r#"{"choices":[{"message":{"content":"","reasoning":"think"}}]}"#;
    let err = parse_completion_body(body).expect_err("empty content");
    assert!(matches!(err, OpenRouterError::MissingContent));
}

#[test]
fn parse_completion_body_reads_reasoning_details() {
    let body = r#"{"choices":[{"message":{"content":"answer","reasoning_details":[{"text":"step1"},{"summary":"step2"}]}}]}"#;
    let resp = parse_completion_body(body).expect("parse");
    assert_eq!(resp.content, "answer");
    assert_eq!(resp.reasoning.as_deref(), Some("step1\nstep2"));
}
