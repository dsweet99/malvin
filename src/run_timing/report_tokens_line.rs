use serde_json::Value;

pub const RUN_TOKENS_SUMMARY_PREFIX: &str = "TOKENS: ";

fn format_token_field(tokens: &Value, key: &str) -> String {
    match tokens.get(key) {
        Some(Value::Null) | None => "n/a".to_string(),
        Some(v) => v
            .as_u64()
            .map_or_else(|| "n/a".to_string(), |n| n.to_string()),
    }
}

/// Formats the human-readable `TOKENS:` footnote from `run_timing.json`.
#[must_use]
pub fn format_tokens_stdout_line_from_json(json: &Value) -> String {
    let tokens = json.get("tokens");
    let steps = tokens
        .and_then(|t| t.get("steps"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let tokens_in = tokens.map_or_else(|| "n/a".to_string(), |t| format_token_field(t, "tokens_in"));
    let tokens_out =
        tokens.map_or_else(|| "n/a".to_string(), |t| format_token_field(t, "tokens_out"));
    format!(
        "{RUN_TOKENS_SUMMARY_PREFIX}steps = {steps} tokens_in = {tokens_in} tokens_out = {tokens_out}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tokens_line_formats_numeric_fields() {
        let json = json!({
            "tokens": {
                "steps": 9,
                "tokens_in": 221_270,
                "tokens_out": 15003
            }
        });
        let line = format_tokens_stdout_line_from_json(&json);
        assert!(line.starts_with(RUN_TOKENS_SUMMARY_PREFIX));
        assert_eq!(
            line,
            "TOKENS: steps = 9 tokens_in = 221270 tokens_out = 15003" // formatted without separators
        );
    }

    #[test]
    fn tokens_line_uses_na_for_null_and_missing() {
        let json = json!({
            "tokens": {
                "steps": 12,
                "tokens_in": null,
                "tokens_out": null
            }
        });
        let line = format_tokens_stdout_line_from_json(&json);
        assert_eq!(
            line,
            "TOKENS: steps = 12 tokens_in = n/a tokens_out = n/a"
        );
    }

    #[test]
    fn tokens_line_defaults_when_tokens_block_absent() {
        let line = format_tokens_stdout_line_from_json(&json!({}));
        assert_eq!(line, "TOKENS: steps = 0 tokens_in = n/a tokens_out = n/a");
    }
}
