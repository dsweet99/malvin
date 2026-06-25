use malvin_mini::ResponseUsage;

use super::RunTiming;

impl RunTiming {
    pub fn record_mini_http_cost(&mut self, usage: &ResponseUsage) {
        match usage.cost {
            Some(c) => self.tx_costs.push(c),
            None if usage.total_tokens.is_some() || usage.prompt_tokens.is_some() => {
                self.unknown_tx_count += 1;
            }
            None => {}
        }
    }
}

fn median_of_sorted(sorted: &[f64]) -> f64 {
    if sorted.is_empty() {
        0.0
    } else {
        sorted[sorted.len() / 2]
    }
}

#[must_use]
pub fn cost_stats(tx_costs: &[f64], unknown_tx_count: u32) -> Option<serde_json::Value> {
    if tx_costs.is_empty() && unknown_tx_count == 0 {
        return None;
    }
    let tx_count = u64::try_from(tx_costs.len()).unwrap_or(u64::MAX);
    let total_cost: f64 = tx_costs.iter().sum();
    let mean_cost_per_tx = if tx_costs.is_empty() {
        0.0
    } else {
        total_cost / f64::from(u32::try_from(tx_costs.len()).unwrap_or(u32::MAX))
    };
    let mut sorted = tx_costs.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(serde_json::json!({
        "tx_count": tx_count,
        "total_cost": total_cost,
        "mean_cost_per_tx": mean_cost_per_tx,
        "median_cost_per_tx": median_of_sorted(&sorted),
        "max_cost_per_tx": sorted.last().copied().unwrap_or(0.0),
        "unknown_tx_count": unknown_tx_count,
    }))
}

pub fn record_mini_http_cost(
    timing: Option<&std::sync::Arc<std::sync::Mutex<RunTiming>>>,
    usage: &ResponseUsage,
) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.record_mini_http_cost(usage);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_stats_exclude_unknown_tx_from_mean_median_max() {
        let mut r = RunTiming::default();
        r.record_mini_http_cost(&ResponseUsage {
            prompt_tokens: Some(1),
            completion_tokens: None,
            total_tokens: Some(1),
            cost: None,
        });
        r.record_mini_http_cost(&ResponseUsage {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            cost: Some(0.01),
        });
        let stats = cost_stats(&r.tx_costs, r.unknown_tx_count).expect("stats");
        assert_eq!(stats["tx_count"], 1);
        assert_eq!(stats["unknown_tx_count"], 1);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn median_of_sorted_handles_empty_and_odd_lengths() {
        assert_eq!(median_of_sorted(&[]), 0.0);
        assert_eq!(median_of_sorted(&[3.0]), 3.0);
        assert_eq!(median_of_sorted(&[1.0, 3.0, 5.0]), 3.0);
        let stats = cost_stats(&[0.01, 0.02, 0.03], 0).expect("stats");
        assert_eq!(stats["tx_count"], 3);
        assert_eq!(stats["median_cost_per_tx"], 0.02);
    }
}
