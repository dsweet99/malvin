use crate::openrouter::serde_types::ChatCompletionResponse;
use crate::openrouter::http_exchange::CompletionWithMeta;
use crate::openrouter::types::CompletionResponse;
use crate::error::{is_prompt_too_long_error, OpenRouterError};

use super::super::provider_error::{provider_fatal_from_body, provider_transport_from_body};
use super::{completion_with_meta, transport_meta};

fn http_body_outcome_with_meta(
    status: u16,
    text: String,
    result: Result<CompletionResponse, OpenRouterError>,
) -> CompletionWithMeta {
    completion_with_meta(result, transport_meta(Some(status), Some(text)))
}

fn provider_envelope_outcome(status: u16, text: String) -> Option<CompletionWithMeta> {
    let err = provider_transport_from_body(&text).or_else(|| provider_fatal_from_body(&text))?;
    Some(http_body_outcome_with_meta(status, text, Err(err)))
}

fn parse_http_body_result(
    status: u16,
    text: &str,
    message_count: usize,
) -> Result<CompletionResponse, OpenRouterError> {
    match map_http_status(status, text) {
        Ok(()) => parse_completion_body(text),
        Err(err) if is_prompt_too_long_error(&err) => Err(OpenRouterError::ContextOverflow {
            body: err.to_string(),
            message_count,
        }),
        Err(err) => Err(err),
    }
}

pub(crate) fn outcome_from_http_body(status: u16, text: String, message_count: usize) -> CompletionWithMeta {
    if let Some(outcome) = provider_envelope_outcome(status, text.clone()) {
        return outcome;
    }
    let result = parse_http_body_result(status, &text, message_count);
    http_body_outcome_with_meta(status, text, result)
}

pub(crate) fn map_http_status(status: u16, body: &str) -> Result<(), OpenRouterError> {
    match status {
        200 => Ok(()),
        401 => Err(OpenRouterError::Unauthorized {
            body: body.to_string(),
        }),
        402 | 403 => Err(OpenRouterError::BillingFailure { status, body: body.to_string() }),
        429 => Err(OpenRouterError::RateLimited {
            body: body.to_string(),
        }),
        500..=599 => Err(OpenRouterError::ServerError {
            status,
            body: body.to_string(),
        }),
        _ => Err(OpenRouterError::RequestFailed {
            status,
            body: body.to_string(),
        }),
    }
}

pub(crate) fn parse_completion_body(text: &str) -> Result<CompletionResponse, OpenRouterError> {
    let mut value: serde_json::Value = serde_json::from_str(text)?;
    normalize_message_content_fields(&mut value);
    let parsed: ChatCompletionResponse = serde_json::from_value(value)?;
    let message = parsed
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .ok_or(OpenRouterError::MissingContent)?;
    let content = message.text_content().ok_or(OpenRouterError::MissingContent)?;
    let reasoning = message.reasoning.clone();
    Ok(CompletionResponse {
        content,
        usage: parsed.usage,
        reasoning,
    })
}

fn normalize_message_content_fields(value: &mut serde_json::Value) {
    let Some(choices) = value.get_mut("choices").and_then(serde_json::Value::as_array_mut) else {
        return;
    };
    for choice in choices {
        let Some(content) = choice.pointer_mut("/message/content") else {
            continue;
        };
        if let Some(normalized) = normalize_content_value(content) {
            *content = normalized;
        } else if content.is_array() {
            *content = serde_json::Value::String(String::new());
        }
    }
}

fn normalize_content_value(content: &serde_json::Value) -> Option<serde_json::Value> {
    match content {
        serde_json::Value::Array(parts) => {
            let joined: Vec<String> = parts
                .iter()
                .filter_map(|part| {
                    part.get("text")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
                .collect();
            if joined.is_empty() {
                None
            } else {
                Some(serde_json::Value::String(joined.join("\n")))
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{map_http_status, outcome_from_http_body, parse_completion_body};
    use crate::error::OpenRouterError;

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
    fn parse_completion_body_falls_back_to_reasoning() {
        let body = r#"{"choices":[{"message":{"content":"","reasoning":"think"}}]}"#;
        let resp = parse_completion_body(body).expect("parse reasoning");
        assert_eq!(resp.content, "think");
    }
}
