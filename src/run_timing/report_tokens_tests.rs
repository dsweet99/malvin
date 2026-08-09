use super::report_timing_line::format_timing_stdout_line_from_json;
use super::to_json_value;
use super::super::report_cost_line::format_cost_stdout_line_from_json;
use crate::malvin_config_file::TokenCostRates;
use crate::llm_transport::ResponseUsage;
use crate::run_timing::RunTiming;

#[test]
fn run_timing_json_includes_tokens_block() {
    let mut r = RunTiming::default();
    r.record_completion_step(Some(&ResponseUsage {
        prompt_tokens: Some(100),
        completion_tokens: Some(20),
        total_tokens: Some(120),
        cost: Some(0.01),
    }));
    let json = to_json_value(&r);
    assert_eq!(json["tokens"]["steps"], 1);
    assert_eq!(json["tokens"]["tokens_in"], 100);
    assert_eq!(json["tokens"]["tokens_out"], 20);
}

#[test]
fn tokens_and_cost_fields_on_combined_cost_line_not_timing_line() {
    let mut r = RunTiming::default();
    r.record_completion_step(Some(&ResponseUsage {
        prompt_tokens: Some(50),
        completion_tokens: Some(10),
        total_tokens: Some(60),
        cost: None,
    }));
    r.record_completion_cost(&ResponseUsage {
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: Some(60),
        cost: Some(0.01),
    });
    let json = to_json_value(&r);
    let timing_line = format_timing_stdout_line_from_json(&json);
    assert!(!timing_line.contains("tokens_in"));
    assert!(!timing_line.contains("steps ="));
    let cost_line = format_cost_stdout_line_from_json(&json);
    assert!(cost_line.starts_with("COST:"));
    assert!(cost_line.contains("steps = 1"));
    assert!(cost_line.contains("tokens_in = 50"));
    assert!(cost_line.contains("tokens_out = 10"));
    assert!(cost_line.contains("cache_read = n/a"));
    assert!(cost_line.contains("cache_write = n/a"));
    assert!(cost_line.contains("cost_tot = 0.0000"));
}

#[test]
fn cost_line_uses_na_when_never_observed() {
    let json = to_json_value(&RunTiming::default());
    let line = format_cost_stdout_line_from_json(&json);
    assert_eq!(
        line,
        "COST: steps = 0 tokens_in = n/a tokens_out = n/a cache_read = n/a cache_write = n/a cost_in = n/a cost_out = n/a cost_read = n/a cost_write = n/a cost_tot = n/a"
    );
}

#[test]
fn cost_line_shows_cache_fields_and_estimated_cost_after_sdk_shaped_usage() {
    let mut r = RunTiming {
        token_cost_rates: TokenCostRates {
            usd_per_microtoken_in: 1000.0,
            usd_per_microtoken_out: 2000.0,
            usd_per_microtoken_cache_read: 100.0,
            usd_per_microtoken_cache_write: 500.0,
        },
        ..Default::default()
    };
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 10,
        "outputTokens": 3,
        "cacheReadTokens": 2,
        "cacheWriteTokens": 1
    }));
    let json = to_json_value(&r);
    assert_eq!(json["tokens"]["cache_read"], 2);
    assert_eq!(json["tokens"]["cache_write"], 1);
    // 10*1000/1e6 + 3*2000/1e6 + 2*100/1e6 + 1*500/1e6 = 0.0167
    assert!((json["cost"]["cost_tot"].as_f64().unwrap() - 0.0167).abs() < 1e-9);
    assert!((json["cost"]["cost_in"].as_f64().unwrap() - 0.01).abs() < 1e-9);
    assert!((json["cost"]["cost_out"].as_f64().unwrap() - 0.006).abs() < 1e-9);
    assert!((json["cost"]["cost_read"].as_f64().unwrap() - 0.0002).abs() < 1e-9);
    assert!((json["cost"]["cost_write"].as_f64().unwrap() - 0.0005).abs() < 1e-9);
    let line = format_cost_stdout_line_from_json(&json);
    assert_eq!(
        line,
        "COST: steps = 0 tokens_in = 13 tokens_out = 3 cache_read = 2 cache_write = 1 cost_in = 0.0100 cost_out = 0.0060 cost_read = 0.0002 cost_write = 0.0005 cost_tot = 0.0167"
    );
}
