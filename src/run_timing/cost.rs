use crate::llm_transport::ResponseUsage;

use super::RunTiming;

impl RunTiming {
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

fn merged_cost_stats(r: &RunTiming, source: &str) -> serde_json::Value {
    let cost_in = r.reported_cost_in.unwrap_or(0.0) + r.estimated_cost_in.unwrap_or(0.0);
    let cost_out = r.reported_cost_out.unwrap_or(0.0) + r.estimated_cost_out.unwrap_or(0.0);
    let cost_read = r.reported_cost_read.unwrap_or(0.0) + r.estimated_cost_read.unwrap_or(0.0);
    let cost_write = r.reported_cost_write.unwrap_or(0.0) + r.estimated_cost_write.unwrap_or(0.0);
    let cost_tot = cost_in + cost_out + cost_read + cost_write;
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
}
