use super::timing::{prime_note_sdk_step, prime_record_sdk_usage};
use std::sync::{Arc, Mutex};
use crate::run_timing::RunTiming;

#[test]
fn note_sdk_step_increments() {
    let t = Arc::new(Mutex::new(RunTiming::default()));
    prime_note_sdk_step(Some(&t));
    assert_eq!(t.lock().unwrap().steps, 1);
}

#[test]
fn record_sdk_usage_folds_cache_into_tokens_in() {
    let t = Arc::new(Mutex::new(RunTiming::default()));
    let usage = serde_json::json!({
        "inputTokens": 11,
        "outputTokens": 3,
        "cacheReadTokens": 2,
        "cacheWriteTokens": 1
    });
    prime_record_sdk_usage(Some(&t), &usage);
    let (tokens_in, tokens_out, cache_read, cache_write) = {
        let g = t.lock().unwrap();
        (g.tokens_in, g.tokens_out, g.cache_read, g.cache_write)
    };
    assert_eq!(tokens_in, Some(14));
    assert_eq!(tokens_out, Some(3));
    assert_eq!(cache_read, Some(2));
    assert_eq!(cache_write, Some(1));
}

#[test]
fn record_sdk_usage_maps_prime_field_names() {
    let t = Arc::new(Mutex::new(RunTiming::default()));
    let usage = serde_json::json!({
        "input": 4722,
        "output": 21,
        "cacheRead": 0,
        "cacheWrite": 0,
        "totalTokens": 4743
    });
    prime_record_sdk_usage(Some(&t), &usage);
    let (tokens_in, tokens_out, usage_tx) = {
        let g = t.lock().unwrap();
        (g.tokens_in, g.tokens_out, g.usage_tx_count)
    };
    assert_eq!(tokens_in, Some(4722));
    assert_eq!(tokens_out, Some(21));
    assert_eq!(usage_tx, 1);
}
