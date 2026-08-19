use serde_json::Value;

pub const RUN_COST_SUMMARY_PREFIX: &str = "COST: ";

fn format_token_field(tokens: &Value, key: &str) -> String {
    match tokens.get(key) {
        Some(Value::Null) | None => "n/a".to_string(),
        Some(v) => v
            .as_u64()
            .map_or_else(|| "n/a".to_string(), |n| n.to_string()),
    }
}

fn format_cost_field(cost: Option<&Value>, key: &str) -> String {
    cost.and_then(|c| c.get(key))
        .and_then(Value::as_f64)
        .map_or_else(|| "n/a".to_string(), |n| format!("{n:.4}"))
}

fn token_fields_fragment(json: &Value) -> String {
    let tokens = json.get("tokens");
    let steps = tokens
        .and_then(|t| t.get("steps"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let tokens_in =
        tokens.map_or_else(|| "n/a".to_string(), |t| format_token_field(t, "tokens_in"));
    let tokens_out = tokens.map_or_else(
        || "n/a".to_string(),
        |t| format_token_field(t, "tokens_out"),
    );
    let cache_read = tokens.map_or_else(
        || "n/a".to_string(),
        |t| format_token_field(t, "cache_read"),
    );
    let cache_write = tokens.map_or_else(
        || "n/a".to_string(),
        |t| format_token_field(t, "cache_write"),
    );
    format!(
        "steps = {steps} tokens_in = {tokens_in} tokens_out = {tokens_out} cache_read = {cache_read} cache_write = {cache_write}"
    )
}

fn cost_fields_fragment(json: &Value) -> String {
    let cost = json.get("cost");
    format!(
        "cost_in = {} cost_out = {} cost_read = {} cost_write = {} cost_tot = {}",
        format_cost_field(cost, "cost_in"),
        format_cost_field(cost, "cost_out"),
        format_cost_field(cost, "cost_read"),
        format_cost_field(cost, "cost_write"),
        format_cost_field(cost, "cost_tot"),
    )
}

#[must_use]
pub fn format_cost_stdout_line_from_json(json: &Value) -> String {
    format!(
        "{RUN_COST_SUMMARY_PREFIX}{} {}",
        token_fields_fragment(json),
        cost_fields_fragment(json)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cost_stdout_line_combines_token_and_cost_fields() {
        let json = json!({
            "tokens": {
                "steps": 9,
                "tokens_in": 221_270,
                "tokens_out": 15003,
                "cache_read": 1200,
                "cache_write": 80
            },
            "cost": {
                "cost_in": 0.05,
                "cost_out": 0.03,
                "cost_read": 0.002,
                "cost_write": 0.0022,
                "cost_tot": 0.0842
            }
        });
        let line = format_cost_stdout_line_from_json(&json);
        assert!(line.starts_with(RUN_COST_SUMMARY_PREFIX));
        assert_eq!(
            line,
            "COST: steps = 9 tokens_in = 221270 tokens_out = 15003 cache_read = 1200 cache_write = 80 cost_in = 0.0500 cost_out = 0.0300 cost_read = 0.0020 cost_write = 0.0022 cost_tot = 0.0842"
        );
    }

    #[test]
    fn cost_stdout_line_uses_na_when_blocks_absent() {
        let line = format_cost_stdout_line_from_json(&json!({}));
        assert_eq!(
            line,
            "COST: steps = 0 tokens_in = n/a tokens_out = n/a cache_read = n/a cache_write = n/a cost_in = n/a cost_out = n/a cost_read = n/a cost_write = n/a cost_tot = n/a"
        );
    }

    #[test]
    fn cost_stdout_line_formats_four_decimal_places() {
        let json = json!({
            "tokens": {
                "steps": 1,
                "tokens_in": 10,
                "tokens_out": 2
            },
            "cost": {
                "cost_in": 0.05,
                "cost_out": 0.03,
                "cost_read": 0.0,
                "cost_write": 0.0,
                "cost_tot": 0.0842
            }
        });
        let line = format_cost_stdout_line_from_json(&json);
        assert!(line.contains("cost_tot = 0.0842"));
        assert!(line.contains("cost_in = 0.0500"));
        assert!(line.contains("cache_read = n/a"));
    }
}
