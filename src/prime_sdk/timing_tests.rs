use crate::bridge_sdk::{note_sdk_step, record_sdk_usage};
use std::sync::{Arc, Mutex};
use crate::run_timing::RunTiming;

#[test]
fn prime_note_sdk_step_increments() {
    let t = Arc::new(Mutex::new(RunTiming::default()));
    note_sdk_step(Some(&t));
    assert_eq!(t.lock().unwrap().steps, 1);
}

#[test]
fn prime_record_sdk_usage_aliases() {
    let t = Arc::new(Mutex::new(RunTiming::default()));
    let usage = serde_json::json!({
        "input": 10,
        "output": 3,
        "cacheRead": 2,
        "cacheWrite": 1
    });
    record_sdk_usage(Some(&t), &usage, true);
    let (tokens_in, tokens_out) = {
        let g = t.lock().unwrap();
        (g.tokens_in, g.tokens_out)
    };
    assert_eq!(tokens_in, Some(13));
    assert_eq!(tokens_out, Some(3));
}
