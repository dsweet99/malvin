use super::*;
use crate::openrouter_transport::ResponseUsage;
use crate::run_timing::RunTiming;

#[test]
fn mini_step_increments_even_without_usage() {
    let mut r = RunTiming::default();
    r.record_mini_http_step(None);
    assert_eq!(r.steps, 1);
    assert_eq!(r.unknown_usage_tx_count, 1);
    assert!(r.tokens_in.is_none());
    assert!(r.tokens_out.is_none());
}

#[test]
fn mini_usage_sums_partial_fields_without_inventing_zeros() {
    let mut r = RunTiming::default();
    r.record_mini_http_step(Some(&ResponseUsage {
        prompt_tokens: Some(10),
        completion_tokens: None,
        total_tokens: Some(10),
        cost: None,
    }));
    r.record_mini_http_step(Some(&ResponseUsage {
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
}

#[test]
fn tokens_stats_null_when_never_observed() {
    let r = RunTiming::default();
    let v = tokens_stats(&r);
    assert_eq!(v["steps"], 0);
    assert!(v["tokens_in"].is_null());
    assert!(v["tokens_out"].is_null());
}
