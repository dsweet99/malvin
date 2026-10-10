use super::super::RunTiming;
use crate::config::malvin_config_file::TokenCostRates;

#[test]
#[allow(clippy::float_cmp)]
fn codex_additive_reasoning_folds_into_tokens_out_and_cost() {
    let mut r = RunTiming {
        token_cost_rates: TokenCostRates {
            usd_per_microtoken_in: 0.0,
            usd_per_microtoken_out: 1000.0,
            usd_per_microtoken_cache_read: 0.0,
            usd_per_microtoken_cache_write: 0.0,
        },
        ..Default::default()
    };
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 4,
        "outputTokens": 2,
        "cacheReadTokens": 1,
        "cacheWriteTokens": 0,
        "reasoningTokens": 3,
        "totalTokens": 10
    }));
    assert_eq!(r.tokens_out, Some(5));
    assert_eq!(r.reasoning_tokens, Some(3));
    let stats = super::super::cost::cost_stats(&r).expect("stats");
    assert!((stats["cost_out"].as_f64().unwrap() - 0.005).abs() < 1e-12);
}

#[test]
#[allow(clippy::float_cmp)]
fn codex_total_keeps_cache_inside_input_tokens() {
    let mut r = RunTiming {
        token_cost_rates: TokenCostRates {
            usd_per_microtoken_in: 1_000_000.0,
            usd_per_microtoken_out: 0.0,
            usd_per_microtoken_cache_read: 1_000_000.0,
            usd_per_microtoken_cache_write: 0.0,
        },
        ..Default::default()
    };
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 35576,
        "outputTokens": 101,
        "cacheReadTokens": 27136,
        "cacheWriteTokens": 0,
        "reasoningTokens": 37,
        "totalTokens": 35677
    }));
    assert_eq!(r.tokens_in, Some(35576));
    assert_eq!(r.cache_read, Some(27136));
    assert_eq!(r.tokens_out, Some(101));
    assert_eq!(r.reasoning_tokens, Some(37));
    let stats = super::super::cost::cost_stats(&r).expect("stats");
    assert!((stats["cost_in"].as_f64().unwrap() - 8440.0).abs() < 1e-6);
    assert!((stats["cost_read"].as_f64().unwrap() - 27136.0).abs() < 1e-6);
}

#[test]
fn cursor_total_still_folds_cache_into_tokens_in() {
    let mut r = RunTiming::default();
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 18844,
        "outputTokens": 54,
        "cacheReadTokens": 8704,
        "cacheWriteTokens": 0,
        "totalTokens": 27602
    }));
    assert_eq!(r.tokens_in, Some(27548));
    assert_eq!(r.cache_read, Some(8704));
}

#[test]
fn cursor_subset_reasoning_does_not_inflate_tokens_out() {
    let mut r = RunTiming::default();
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 100,
        "outputTokens": 50,
        "cacheReadTokens": 0,
        "cacheWriteTokens": 0,
        "reasoningTokens": 10,
        "totalTokens": 150
    }));
    assert_eq!(r.tokens_out, Some(50));
    assert_eq!(r.reasoning_tokens, Some(10));
}

#[test]
#[allow(clippy::float_cmp)]
fn total_only_reported_cost_keeps_billed_total() {
    let mut r = RunTiming {
        token_cost_rates: TokenCostRates {
            usd_per_microtoken_in: 1_000_000.0,
            usd_per_microtoken_out: 1_000_000.0,
            usd_per_microtoken_cache_read: 1_000_000.0,
            usd_per_microtoken_cache_write: 0.0,
        },
        ..Default::default()
    };
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 1000,
        "outputTokens": 100,
        "cacheReadTokens": 500,
        "costUsd": {
            "input": 0.0,
            "output": 0.0,
            "cacheRead": 0.0,
            "cacheWrite": 0.0,
            "total": 0.03694
        }
    }));
    let stats = super::super::cost::cost_stats(&r).expect("stats");
    assert_eq!(stats["source"], "reported");
    assert!((stats["cost_in"].as_f64().unwrap()).abs() < 1e-12);
    assert!((stats["cost_tot"].as_f64().unwrap() - 0.03694).abs() < 1e-12);
}

#[test]
#[allow(clippy::float_cmp)]
fn reported_cost_usd_skips_rate_estimate() {
    let mut r = RunTiming {
        token_cost_rates: TokenCostRates {
            usd_per_microtoken_in: 1_000_000.0,
            usd_per_microtoken_out: 1_000_000.0,
            usd_per_microtoken_cache_read: 0.0,
            usd_per_microtoken_cache_write: 0.0,
        },
        ..Default::default()
    };
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 10,
        "outputTokens": 2,
        "costUsd": {
            "input": 0.01,
            "output": 0.02,
            "cacheRead": 0.0,
            "cacheWrite": 0.0,
            "total": 0.03
        }
    }));
    assert_eq!(r.tx_costs, vec![0.03]);
    let stats = super::super::cost::cost_stats(&r).expect("stats");
    assert_eq!(stats["source"], "reported");
    assert!((stats["cost_tot"].as_f64().unwrap() - 0.03).abs() < 1e-12);
}

#[test]
#[allow(clippy::float_cmp)]
fn mixed_reported_and_estimated_turns_merge_cost_components() {
    let mut r = RunTiming {
        token_cost_rates: TokenCostRates {
            usd_per_microtoken_in: 1_000_000.0,
            usd_per_microtoken_out: 0.0,
            usd_per_microtoken_cache_read: 0.0,
            usd_per_microtoken_cache_write: 0.0,
        },
        ..Default::default()
    };
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 10,
        "outputTokens": 0,
        "costUsd": { "input": 0.01, "output": 0.0, "total": 0.01 }
    }));
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 10,
        "outputTokens": 0
    }));
    assert_eq!(r.tx_costs, vec![0.01, 10.0]);
    let stats = super::super::cost::cost_stats(&r).expect("stats");
    assert_eq!(stats["source"], "mixed");
    assert!((stats["cost_in"].as_f64().unwrap() - 10.01).abs() < 1e-12);
    assert!((stats["cost_tot"].as_f64().unwrap() - 10.01).abs() < 1e-12);
}
