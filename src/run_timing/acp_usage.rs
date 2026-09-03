use super::RunTiming;

pub(super) struct AcpUsageFields {
    pub input: Option<u64>,
    pub output: Option<u64>,
    pub cache_read: Option<u64>,
    pub cache_write: Option<u64>,
    pub reasoning: Option<u64>,
    pub total: Option<u64>,
    pub skip_rate_estimate: bool,
}

pub(super) struct ReportedCostUsd {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    pub total: f64,
}

pub(super) fn u64_field(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<u64> {
    obj.get(key).and_then(serde_json::Value::as_u64)
}

fn f64_field(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<f64> {
    obj.get(key).and_then(serde_json::Value::as_f64)
}

pub(super) fn add_optional_sum(slot: &mut Option<u64>, n: u64) {
    *slot = Some(slot.unwrap_or(0).saturating_add(n));
}

fn add_optional_f64_sum(slot: &mut Option<f64>, n: f64) {
    *slot = Some(slot.unwrap_or(0.0) + n);
}

/// When `totalTokens` equals the sum including `reasoningTokens` but not without it,
/// reasoning is billed separately from `outputTokens` (Codex). Cursor reports reasoning
/// as a subset of output, so the totals do not match that pattern.
pub(super) fn reasoning_is_additive(fields: &AcpUsageFields) -> bool {
    let Some(total) = fields.total else {
        return false;
    };
    let reasoning = fields.reasoning.unwrap_or(0);
    if reasoning == 0 {
        return false;
    }
    let input = fields.input.unwrap_or(0);
    let output = fields.output.unwrap_or(0);
    let cache_read = fields.cache_read.unwrap_or(0);
    let cache_write = fields.cache_write.unwrap_or(0);
    let with = input
        .saturating_add(output)
        .saturating_add(cache_read)
        .saturating_add(cache_write)
        .saturating_add(reasoning);
    let without = input
        .saturating_add(output)
        .saturating_add(cache_read)
        .saturating_add(cache_write);
    with == total && with != without
}

pub(super) fn usage_payload_is_observable(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> bool {
    u64_field(obj, "inputTokens").is_some()
        || u64_field(obj, "outputTokens").is_some()
        || u64_field(obj, "cacheReadTokens").is_some()
        || u64_field(obj, "cacheWriteTokens").is_some()
        || u64_field(obj, "reasoningTokens").is_some()
        || reported_cost_usd(obj).is_some()
}

pub(super) fn reported_cost_usd(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Option<ReportedCostUsd> {
    let cost = obj.get("costUsd")?.as_object()?;
    let input = f64_field(cost, "input").unwrap_or(0.0);
    let output = f64_field(cost, "output").unwrap_or(0.0);
    let cache_read = f64_field(cost, "cacheRead").unwrap_or(0.0);
    let cache_write = f64_field(cost, "cacheWrite").unwrap_or(0.0);
    let total = f64_field(cost, "total").unwrap_or(input + output + cache_read + cache_write);
    if total == 0.0 && input == 0.0 && output == 0.0 && cache_read == 0.0 && cache_write == 0.0 {
        return None;
    }
    Some(ReportedCostUsd {
        input,
        output,
        cache_read,
        cache_write,
        total,
    })
}

impl RunTiming {
    pub(super) fn record_reported_cost_usd(&mut self, cost: &ReportedCostUsd) {
        add_optional_f64_sum(&mut self.reported_cost_in, cost.input);
        add_optional_f64_sum(&mut self.reported_cost_out, cost.output);
        add_optional_f64_sum(&mut self.reported_cost_read, cost.cache_read);
        add_optional_f64_sum(&mut self.reported_cost_write, cost.cache_write);
        self.tx_costs.push(cost.total);
    }

    pub(super) fn apply_acp_usage_fields(&mut self, fields: AcpUsageFields) {
        self.usage_tx_count = self.usage_tx_count.saturating_add(1);
        let input_n = fields.input.unwrap_or(0);
        let output_n = fields.output.unwrap_or(0);
        let cache_read_n = fields.cache_read.unwrap_or(0);
        let cache_write_n = fields.cache_write.unwrap_or(0);
        let reasoning_n = fields.reasoning.unwrap_or(0);
        let reasoning_additive = reasoning_is_additive(&fields);
        let output_for_totals = if reasoning_additive {
            output_n.saturating_add(reasoning_n)
        } else {
            output_n
        };
        let tokens_in = input_n + cache_read_n + cache_write_n;
        if fields.input.is_some() || fields.cache_read.is_some() || fields.cache_write.is_some() {
            add_optional_sum(&mut self.tokens_in, tokens_in);
        }
        if let Some(n) = fields.cache_read {
            add_optional_sum(&mut self.cache_read, n);
        }
        if let Some(n) = fields.cache_write {
            add_optional_sum(&mut self.cache_write, n);
        }
        if fields.output.is_some() || reasoning_additive {
            add_optional_sum(&mut self.tokens_out, output_for_totals);
        }
        if let Some(n) = fields.reasoning.filter(|_| reasoning_n > 0) {
            add_optional_sum(&mut self.reasoning_tokens, n);
        }
        self.record_acp_usage_rate_estimate(
            fields.skip_rate_estimate,
            input_n,
            output_for_totals,
            (cache_read_n, cache_write_n),
        );
    }

    fn record_acp_usage_rate_estimate(
        &mut self,
        skip_rate_estimate: bool,
        input_n: u64,
        output_n: u64,
        cache_n: (u64, u64),
    ) {
        let (cache_read_n, cache_write_n) = cache_n;
        if !skip_rate_estimate && matches!(self.cost_policy, super::CostPolicy::EstimateFromRates) {
            let (cost_in, cost_out, cost_read, cost_write) = self
                .token_cost_rates
                .estimate_components(input_n, output_n, cache_read_n, cache_write_n);
            add_optional_f64_sum(&mut self.estimated_cost_in, cost_in);
            add_optional_f64_sum(&mut self.estimated_cost_out, cost_out);
            add_optional_f64_sum(&mut self.estimated_cost_read, cost_read);
            add_optional_f64_sum(&mut self.estimated_cost_write, cost_write);
            let estimated =
                self.token_cost_rates
                    .estimate_usd(input_n, output_n, cache_read_n, cache_write_n);
            self.tx_costs.push(estimated);
        } else if matches!(self.cost_policy, super::CostPolicy::Zero) {
            self.tx_costs.push(0.0);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn acp_usage_field_structs_round_trip_values() {
        let fields = AcpUsageFields {
            input: Some(4),
            output: Some(2),
            cache_read: Some(1),
            cache_write: Some(0),
            reasoning: Some(3),
            total: Some(10),
            skip_rate_estimate: true,
        };
        assert!(reasoning_is_additive(&fields));
        assert!(fields.skip_rate_estimate);
        let cost = ReportedCostUsd {
            input: 0.1,
            output: 0.2,
            cache_read: 0.01,
            cache_write: 0.02,
            total: 0.33,
        };
        assert!((cost.total - 0.33).abs() < f64::EPSILON);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn reported_cost_usd_parses_optional_cache_fields() {
        let obj = serde_json::json!({
            "costUsd": {
                "input": 0.1,
                "output": 0.2,
                "total": 0.3
            }
        });
        let cost = reported_cost_usd(obj.as_object().expect("obj")).expect("cost");
        assert_eq!(cost.cache_read, 0.0);
        assert_eq!(cost.cache_write, 0.0);
        assert_eq!(cost.total, 0.3);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn reported_cost_usd_accepts_total_only_payload() {
        let obj = serde_json::json!({
            "costUsd": {
                "total": 0.0042
            }
        });
        let cost = reported_cost_usd(obj.as_object().expect("obj")).expect("cost");
        assert_eq!(cost.total, 0.0042);
        assert_eq!(cost.input, 0.0);
        assert_eq!(cost.output, 0.0);
    }

    #[test]
    fn usage_payload_is_observable_rejects_empty_object() {
        let obj = serde_json::Map::new();
        assert!(!usage_payload_is_observable(&obj));
    }

    #[test]
    fn usage_payload_is_observable_accepts_token_fields() {
        let obj = serde_json::json!({ "inputTokens": 1 })
            .as_object()
            .expect("obj")
            .clone();
        assert!(usage_payload_is_observable(&obj));
    }
}

#[cfg(test)]
#[path = "acp_usage_tests.rs"]
mod acp_usage_tests;
