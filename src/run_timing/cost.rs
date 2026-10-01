use crate::llm_transport::ResponseUsage;

use super::RunTiming;

fn carry_u64(slot: &mut Option<u64>, prior: Option<u64>) {
    let Some(n) = prior else {
        return;
    };
    *slot = Some(slot.unwrap_or(0).saturating_add(n));
}

fn carry_f64(slot: &mut Option<f64>, prior: Option<f64>) {
    let Some(n) = prior else {
        return;
    };
    *slot = Some(slot.unwrap_or(0.0) + n);
}

impl RunTiming {
    pub(crate) fn carry_token_and_cost_from(&mut self, prior: &Self) {
        carry_u64(&mut self.tokens_in, prior.tokens_in);
        carry_u64(&mut self.tokens_out, prior.tokens_out);
        carry_u64(&mut self.cache_read, prior.cache_read);
        carry_u64(&mut self.cache_write, prior.cache_write);
        carry_u64(&mut self.reasoning_tokens, prior.reasoning_tokens);
        carry_f64(&mut self.reported_cost_in, prior.reported_cost_in);
        carry_f64(&mut self.reported_cost_out, prior.reported_cost_out);
        carry_f64(&mut self.reported_cost_read, prior.reported_cost_read);
        carry_f64(&mut self.reported_cost_write, prior.reported_cost_write);
        carry_f64(&mut self.reported_cost_total, prior.reported_cost_total);
        carry_f64(&mut self.estimated_cost_in, prior.estimated_cost_in);
        carry_f64(&mut self.estimated_cost_out, prior.estimated_cost_out);
        carry_f64(&mut self.estimated_cost_read, prior.estimated_cost_read);
        carry_f64(&mut self.estimated_cost_write, prior.estimated_cost_write);
        self.tx_costs.extend_from_slice(&prior.tx_costs);
        self.unknown_tx_count = self.unknown_tx_count.saturating_add(prior.unknown_tx_count);
        self.usage_tx_count = self.usage_tx_count.saturating_add(prior.usage_tx_count);
        self.unknown_usage_tx_count = self
            .unknown_usage_tx_count
            .saturating_add(prior.unknown_usage_tx_count);
        self.steps = self.steps.saturating_add(prior.steps);
    }

    pub fn record_completion_cost(&mut self, usage: &ResponseUsage) {
        if matches!(self.cost_policy, super::CostPolicy::Zero) {
            return;
        }
        match usage.cost {
            Some(c) => self.tx_costs.push(c),
            None if usage.total_tokens.is_some() || usage.prompt_tokens.is_some() => {
                self.unknown_tx_count += 1;
            }
            None => {}
        }
    }
}

fn input_tokens_for_cost(r: &RunTiming) -> u64 {
    let tokens_in = r.tokens_in.unwrap_or(0);
    let cache_read = r.cache_read.unwrap_or(0);
    let cache_write = r.cache_write.unwrap_or(0);
    tokens_in
        .saturating_sub(cache_read)
        .saturating_sub(cache_write)
}

const fn has_reported_cost_components(r: &RunTiming) -> bool {
    r.reported_cost_in.is_some()
        || r.reported_cost_out.is_some()
        || r.reported_cost_read.is_some()
        || r.reported_cost_write.is_some()
}

const fn has_estimated_cost_components(r: &RunTiming) -> bool {
    r.estimated_cost_in.is_some()
        || r.estimated_cost_out.is_some()
        || r.estimated_cost_read.is_some()
        || r.estimated_cost_write.is_some()
}

const fn has_cost_observation(r: &RunTiming) -> bool {
    r.tokens_in.is_some()
        || r.tokens_out.is_some()
        || r.cache_read.is_some()
        || r.cache_write.is_some()
        || r.reasoning_tokens.is_some()
        || !r.tx_costs.is_empty()
        || has_reported_cost_components(r)
        || has_estimated_cost_components(r)
        || r.unknown_tx_count > 0
}

fn estimated_component_sum(r: &RunTiming) -> f64 {
    r.estimated_cost_in.unwrap_or(0.0)
        + r.estimated_cost_out.unwrap_or(0.0)
        + r.estimated_cost_read.unwrap_or(0.0)
        + r.estimated_cost_write.unwrap_or(0.0)
}

fn merged_total(r: &RunTiming, component_sum: f64) -> f64 {
    match r.reported_cost_total {
        Some(total) if total > 0.0 => total + estimated_component_sum(r),
        _ => component_sum,
    }
}

fn merged_cost_stats(r: &RunTiming, source: &str) -> serde_json::Value {
    let cost_in = r.reported_cost_in.unwrap_or(0.0) + r.estimated_cost_in.unwrap_or(0.0);
    let cost_out = r.reported_cost_out.unwrap_or(0.0) + r.estimated_cost_out.unwrap_or(0.0);
    let cost_read = r.reported_cost_read.unwrap_or(0.0) + r.estimated_cost_read.unwrap_or(0.0);
    let cost_write = r.reported_cost_write.unwrap_or(0.0) + r.estimated_cost_write.unwrap_or(0.0);
    let cost_tot = merged_total(r, cost_in + cost_out + cost_read + cost_write);
    let tx_count = u64::try_from(r.tx_costs.len()).unwrap_or(u64::MAX);
    serde_json::json!({
        "cost_in": cost_in,
        "cost_out": cost_out,
        "cost_read": cost_read,
        "cost_write": cost_write,
        "cost_tot": cost_tot,
        "tx_count": tx_count,
        "unknown_tx_count": r.unknown_tx_count,
        "source": source,
    })
}

#[must_use]
pub fn cost_stats(r: &RunTiming) -> Option<serde_json::Value> {
    if !has_cost_observation(r) {
        return None;
    }
    if has_reported_cost_components(r) {
        let source = if has_estimated_cost_components(r) {
            "mixed"
        } else {
            "reported"
        };
        return Some(merged_cost_stats(r, source));
    }
    let (cost_in, cost_out, cost_read, cost_write) = r.token_cost_rates.estimate_components(
        input_tokens_for_cost(r),
        r.tokens_out.unwrap_or(0),
        r.cache_read.unwrap_or(0),
        r.cache_write.unwrap_or(0),
    );
    let cost_tot = cost_in + cost_out + cost_read + cost_write;
    let tx_count = u64::try_from(r.tx_costs.len()).unwrap_or(u64::MAX);
    Some(serde_json::json!({
        "cost_in": cost_in,
        "cost_out": cost_out,
        "cost_read": cost_read,
        "cost_write": cost_write,
        "cost_tot": cost_tot,
        "tx_count": tx_count,
        "unknown_tx_count": r.unknown_tx_count,
    }))
}

pub fn record_completion_cost(
    timing: Option<&std::sync::Arc<std::sync::Mutex<RunTiming>>>,
    usage: &ResponseUsage,
) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.record_completion_cost(usage);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::malvin_config_file::TokenCostRates;

    #[test]
    fn cost_stats_include_unknown_tx_metadata() {
        let mut r = RunTiming::default();
        r.record_completion_cost(&ResponseUsage {
            prompt_tokens: Some(1),
            completion_tokens: None,
            total_tokens: Some(1),
            cost: None,
        });
        r.record_completion_cost(&ResponseUsage {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            cost: Some(0.01),
        });
        let stats = cost_stats(&r).expect("stats");
        assert_eq!(stats["tx_count"], 1);
        assert_eq!(stats["unknown_tx_count"], 1);
        assert_eq!(stats["cost_tot"], 0.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn cost_stats_rate_times_tokens_components() {
        let mut r = RunTiming {
            token_cost_rates: TokenCostRates {
                usd_per_microtoken_in: 1000.0,
                usd_per_microtoken_out: 2000.0,
                usd_per_microtoken_cache_read: 100.0,
                usd_per_microtoken_cache_write: 500.0,
            },
            ..Default::default()
        };
        r.tokens_in = Some(13);
        r.tokens_out = Some(3);
        r.cache_read = Some(2);
        r.cache_write = Some(1);
        let stats = cost_stats(&r).expect("stats");
        assert!((stats["cost_in"].as_f64().unwrap() - 0.01).abs() < 1e-12);
        assert!((stats["cost_out"].as_f64().unwrap() - 0.006).abs() < 1e-12);
        assert!((stats["cost_read"].as_f64().unwrap() - 0.0002).abs() < 1e-12);
        assert!((stats["cost_write"].as_f64().unwrap() - 0.0005).abs() < 1e-12);
        assert!((stats["cost_tot"].as_f64().unwrap() - 0.0167).abs() < 1e-12);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn cost_stats_prefers_reported_usd_components() {
        let r = RunTiming {
            reported_cost_in: Some(0.01),
            reported_cost_out: Some(0.02),
            reported_cost_read: Some(0.001),
            reported_cost_write: Some(0.0),
            tx_costs: vec![0.031],
            ..Default::default()
        };
        let stats = cost_stats(&r).expect("stats");
        assert_eq!(stats["source"], "reported");
        assert!((stats["cost_tot"].as_f64().unwrap() - 0.031).abs() < 1e-12);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn cost_stats_merges_reported_and_estimated_components() {
        let r = RunTiming {
            reported_cost_in: Some(0.01),
            estimated_cost_in: Some(10.0),
            tx_costs: vec![0.01, 10.0],
            ..Default::default()
        };
        let stats = cost_stats(&r).expect("stats");
        assert_eq!(stats["source"], "mixed");
        assert!((stats["cost_in"].as_f64().unwrap() - 10.01).abs() < 1e-12);
        assert!((stats["cost_tot"].as_f64().unwrap() - 10.01).abs() < 1e-12);
    }

    #[test]
    fn cost_stats_none_when_never_observed() {
        assert!(cost_stats(&RunTiming::default()).is_none());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn later_session_keeps_earlier_reported_cost() {
        let mut slot = None;
        let first =
            super::super::lifecycle::attach_new_run_timing(&mut slot, "rpi:openrouter/x-ai/grok-4.6");
        first.lock().unwrap().record_acp_usage_if_present(&serde_json::json!({
            "inputTokens": 100,
            "outputTokens": 10,
            "cacheReadTokens": 40,
            "costUsd": {
                "input": 0.02,
                "output": 0.01,
                "cacheRead": 0.004,
                "cacheWrite": 0.0,
                "total": 0.034
            }
        }));
        let second =
            super::super::lifecycle::attach_new_run_timing(&mut slot, "rpi:openrouter/x-ai/grok-4.6");
        second.lock().unwrap().record_acp_usage_if_present(&serde_json::json!({
            "inputTokens": 50,
            "outputTokens": 5,
            "cacheReadTokens": 10,
            "costUsd": {
                "input": 0.01,
                "output": 0.005,
                "cacheRead": 0.001,
                "cacheWrite": 0.0,
                "total": 0.016
            }
        }));
        let stats = cost_stats(&second.lock().unwrap()).expect("stats");
        assert!((stats["cost_in"].as_f64().unwrap() - 0.03).abs() < 1e-12);
        assert!((stats["cost_out"].as_f64().unwrap() - 0.015).abs() < 1e-12);
        assert!((stats["cost_read"].as_f64().unwrap() - 0.005).abs() < 1e-12);
        assert!((stats["cost_tot"].as_f64().unwrap() - 0.05).abs() < 1e-12);
        assert_eq!(stats["tx_count"], 2);
        let (tokens_out, cache_read) = {
            let held = second.lock().unwrap();
            (held.tokens_out, held.cache_read)
        };
        assert_eq!(tokens_out, Some(15));
        assert_eq!(cache_read, Some(50));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn billed_total_wins_when_components_are_larger() {
        let r = RunTiming {
            reported_cost_in: Some(10.0),
            reported_cost_total: Some(0.04),
            tx_costs: vec![0.04],
            ..Default::default()
        };
        let stats = cost_stats(&r).expect("stats");
        assert!((stats["cost_in"].as_f64().unwrap() - 10.0).abs() < 1e-12);
        assert!((stats["cost_tot"].as_f64().unwrap() - 0.04).abs() < 1e-12);
    }
}
