use crate::openrouter_transport::serde_types::ChatCompletionResponse;
use crate::openrouter_transport::http_exchange::CompletionWithMeta;
use crate::openrouter_transport::types::CompletionResponse;
use crate::openrouter_transport::error::{body_indicates_prompt_too_long, is_prompt_too_long_error, TransportError};

use super::provider_error::{provider_fatal_from_body, provider_transport_from_body};
use super::complete::{completion_with_meta, transport_meta};

fn http_body_outcome_with_meta(
    status: u16,
    text: String,
    result: Result<CompletionResponse, TransportError>,
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
) -> Result<CompletionResponse, TransportError> {
    match map_http_status(status, text) {
        Ok(()) => parse_completion_body(text),
        Err(err) if is_prompt_too_long_error(&err) => Err(TransportError::ContextOverflow {
            body: err.to_string(),
            message_count,
        }),
        Err(err) => Err(err),
    }
}

pub(crate) fn outcome_from_http_body(status: u16, text: String, message_count: usize) -> CompletionWithMeta {
    // Prompt-budget overflows must become ContextOverflow *before* provider envelopes
    // classify them as fatal ProviderError (which skips session shrink recovery).
    if body_indicates_prompt_too_long(&text) {
        return http_body_outcome_with_meta(
            status,
            text.clone(),
            Err(TransportError::ContextOverflow {
                body: text,
                message_count,
            }),
        );
    }
    if let Some(outcome) = provider_envelope_outcome(status, text.clone()) {
        return outcome;
    }
    let result = parse_http_body_result(status, &text, message_count);
    http_body_outcome_with_meta(status, text, result)
}

pub(crate) fn map_http_status(status: u16, body: &str) -> Result<(), TransportError> {
    match status {
        200 => Ok(()),
        401 => Err(TransportError::Unauthorized {
            body: body.to_string(),
        }),
        402 | 403 => Err(TransportError::BillingFailure { status, body: body.to_string() }),
        429 => Err(TransportError::RateLimited {
            body: body.to_string(),
        }),
        500..=599 => Err(TransportError::ServerError {
            status,
            body: body.to_string(),
        }),
        _ => Err(TransportError::RequestFailed {
            status,
            body: body.to_string(),
        }),
    }
}

pub(crate) fn parse_completion_body(text: &str) -> Result<CompletionResponse, TransportError> {
    let mut value: serde_json::Value = serde_json::from_str(text)?;
    normalize_message_content_fields(&mut value);
    let parsed: ChatCompletionResponse = serde_json::from_value(value)?;
    let message = parsed
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .ok_or(TransportError::MissingContent)?;
    let content = message.text_content().ok_or(TransportError::MissingContent)?;
    let reasoning = message.reasoning_text();
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

