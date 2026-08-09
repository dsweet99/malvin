use super::*;
use crate::llm_transport::ResponseUsage;
use crate::run_timing::{CostPolicy, RunTiming};

#[test]
fn mini_step_increments_even_without_usage() {
    let mut r = RunTiming::default();
    r.record_completion_step(None);
    assert_eq!(r.steps, 1);
    assert_eq!(r.unknown_usage_tx_count, 1);
    assert!(r.tokens_in.is_none());
    assert!(r.tokens_out.is_none());
}

#[test]
fn mini_usage_sums_partial_fields_without_inventing_zeros() {
    let mut r = RunTiming::default();
    r.record_completion_step(Some(&ResponseUsage {
        prompt_tokens: Some(10),
        completion_tokens: None,
        total_tokens: Some(10),
        cost: None,
    }));
    r.record_completion_step(Some(&ResponseUsage {
        prompt_tokens: None,
        completion_tokens: Some(5),
        total_tokens: None,
        cost: None,
    }));
    assert_eq!(r.steps, 2);
    assert_eq!(r.tokens_in, Some(10));
    assert_eq!(r.tokens_out, Some(5));
    assert_eq!(r.usage_tx_count, 2);
}

#[test]
fn acp_parallel_tool_starts_count_as_one_step() {
    let mut r = RunTiming::default();
    r.note_acp_tool_call_start();
    r.note_acp_tool_call_start();
    r.note_acp_tool_call_start();
    r.note_acp_tool_call_completion();
    r.note_acp_tool_call_completion();
    r.note_acp_tool_call_completion();
    assert_eq!(r.steps, 1);
    assert_eq!(r.tool_call_starts, 3);
}

#[test]
fn acp_sequential_batches_count_separately() {
    let mut r = RunTiming::default();
    r.note_acp_tool_call_start();
    r.note_acp_tool_call_completion();
    r.note_acp_tool_call_start();
    r.note_acp_tool_call_completion();
    assert_eq!(r.steps, 2);
}

#[test]
fn acp_trailing_assistant_after_batch_adds_step_on_finalize() {
    let mut r = RunTiming::default();
    r.note_acp_tool_call_start();
    r.note_acp_tool_call_completion();
    r.note_acp_assistant_activity();
    assert_eq!(r.steps, 1);
    r.finalize_acp_trailing_assistant_step();
    assert_eq!(r.steps, 2);
}

#[test]
fn acp_assistant_then_tools_is_one_step() {
    let mut r = RunTiming::default();
    r.note_acp_assistant_activity();
    r.note_acp_tool_call_start();
    r.note_acp_tool_call_completion();
    r.finalize_acp_trailing_assistant_step();
    assert_eq!(r.steps, 1);
}

#[test]
fn acp_usage_folds_cache_into_tokens_in() {
    let mut r = RunTiming::default();
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 100,
        "outputTokens": 20,
        "cacheReadTokens": 50,
        "cacheWriteTokens": 5
    }));
    assert_eq!(r.tokens_in, Some(155));
    assert_eq!(r.tokens_out, Some(20));
    assert_eq!(r.cache_read, Some(50));
    assert_eq!(r.cache_write, Some(5));
    assert_eq!(
        r.tx_costs,
        vec![0.0],
        "EstimateFromRates with zero rates still records an estimated 0 cost row"
    );
    let stats = tokens_stats(&r);
    assert_eq!(stats["cache_read"], 50);
    assert_eq!(stats["cache_write"], 5);
}

#[test]
#[allow(clippy::float_cmp)]
fn cursor_estimate_policy_records_zero_cost_when_rates_unset() {
    let mut r = RunTiming {
        cost_policy: CostPolicy::EstimateFromRates,
        ..Default::default()
    };
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 10,
        "outputTokens": 3,
        "cacheReadTokens": 2,
        "cacheWriteTokens": 1
    }));
    assert_eq!(r.tx_costs, vec![0.0]);
    let stats = super::super::cost::cost_stats(&r).expect("stats");
    assert_eq!(stats["cost_tot"], 0.0);
    let line = super::super::report_cost_line::format_cost_stdout_line_from_json(
        &serde_json::json!({ "tokens": tokens_stats(&r), "cost": stats }),
    );
    assert!(line.contains("cost_tot = 0.0000"));
    assert!(!line.contains("cost_tot = n/a"));
}

#[test]
#[allow(clippy::float_cmp)]
fn zero_policy_records_zero_cost_from_acp_usage() {
    let mut r = RunTiming {
        cost_policy: CostPolicy::Zero,
        ..Default::default()
    };
    r.record_acp_usage_if_present(&serde_json::json!({
        "inputTokens": 10,
        "outputTokens": 3,
        "cacheReadTokens": 0,
        "cacheWriteTokens": 0
    }));
    assert_eq!(r.tx_costs, vec![0.0]);
}

#[test]
fn cost_policy_for_model_maps_prefixes() {
    assert_eq!(
        crate::run_timing::cost_policy_for_model("prime:local/qwen35_9b_q4"),
        CostPolicy::Zero
    );
    assert_eq!(
        crate::run_timing::cost_policy_for_model("cursor:auto"),
        CostPolicy::EstimateFromRates
    );
    assert_eq!(
        crate::run_timing::cost_policy_for_model("prime:openrouter/org/model"),
        CostPolicy::EstimateFromRates
    );
    assert_eq!(
        crate::run_timing::cost_policy_for_model("prime:openai/gpt-5.5"),
        CostPolicy::EstimateFromRates
    );
}

#[test]
fn tokens_stats_null_when_never_observed() {
    let r = RunTiming::default();
    let v = tokens_stats(&r);
    assert_eq!(v["steps"], 0);
    assert!(v["tokens_in"].is_null());
    assert!(v["tokens_out"].is_null());
}

#[test]
#[allow(clippy::float_cmp)]
fn local_cost_policy_forces_zero_per_step() {
    let mut r = RunTiming {
        cost_policy: CostPolicy::Zero,
        ..Default::default()
    };
    r.record_completion_step(None);
    r.record_completion_step(Some(&ResponseUsage {
        prompt_tokens: Some(3),
        completion_tokens: Some(1),
        total_tokens: Some(4),
        cost: None,
    }));
    // Even if a caller also invokes cost recording, Zero policy ignores returned/missing cost.
    r.record_completion_cost(&ResponseUsage {
        prompt_tokens: Some(3),
        completion_tokens: Some(1),
        total_tokens: Some(4),
        cost: None,
    });
    assert_eq!(r.tx_costs, vec![0.0, 0.0]);
    assert_eq!(r.unknown_tx_count, 0);
}

#[test]
#[allow(clippy::float_cmp)]
fn openrouter_cost_policy_uses_reported_cost_only() {
    let mut r = RunTiming {
        cost_policy: CostPolicy::EstimateFromRates,
        ..Default::default()
    };
    r.record_completion_step(Some(&ResponseUsage {
        prompt_tokens: Some(10),
        completion_tokens: Some(2),
        total_tokens: Some(12),
        cost: Some(0.0042),
    }));
    r.record_completion_cost(&ResponseUsage {
        prompt_tokens: Some(10),
        completion_tokens: Some(2),
        total_tokens: Some(12),
        cost: Some(0.0042),
    });
    assert_eq!(r.tx_costs, vec![0.0042]);
}
