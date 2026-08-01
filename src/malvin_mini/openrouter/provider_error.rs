use crate::malvin_mini::error::OpenRouterError;

/// When OpenRouter returns a provider-side capacity error (e.g. Nvidia
/// `ResourceExhausted`), surface `{provider}: {detail}` and retry as transport.
pub(crate) fn provider_transport_from_body(body: &str) -> Option<OpenRouterError> {
    let (provider, error_type, top_message, raw_detail) = parse_provider_error_envelope(body)?;
    let detail = select_provider_detail(&top_message, &raw_detail);
    if is_credit_affordability_error(&detail, &raw_detail) {
        return None;
    }
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
    let detail = select_provider_detail(&top_message, &raw_detail);
    if is_credit_affordability_error(&detail, &raw_detail) {
        return Some(OpenRouterError::BillingFailure {
            status: 402,
            body: detail,
        });
    }
    if is_provider_transport_retryable(&error_type, &top_message, &raw_detail) {
        return None;
    }
    Some(OpenRouterError::ProviderError { provider, detail })
}

fn is_credit_affordability_error(detail: &str, raw_detail: &str) -> bool {
    let combined = format!("{detail} {raw_detail}").to_ascii_lowercase();
    (combined.contains("more credits") && combined.contains("max_tokens"))
        || combined.contains("requires more credits")
        || combined.contains("can only afford")
        // Account-empty / balance-zero copy (not max_tokens affordability).
        || combined.contains("insufficient credits")
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

pub(crate) fn select_provider_detail(top_message: &str, raw_detail: &str) -> String {
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

pub(crate) fn extract_raw_message(raw: &serde_json::Value) -> String {
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

