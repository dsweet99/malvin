use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::run_timing::RunTiming;

pub fn note_sdk_step(timing: Option<&Arc<Mutex<RunTiming>>>) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.steps = g.steps.saturating_add(1);
}

pub fn record_sdk_usage(timing: Option<&Arc<Mutex<RunTiming>>>, usage: &Value) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.record_acp_usage_if_present(usage);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_sdk_step_increments() {
        let t = Arc::new(Mutex::new(RunTiming::default()));
        note_sdk_step(Some(&t));
        assert_eq!(t.lock().unwrap().steps, 1);
    }

    #[test]
    fn record_sdk_usage_folds_cache_into_tokens_in() {
        let t = Arc::new(Mutex::new(RunTiming::default()));
        let usage = serde_json::json!({
            "inputTokens": 10,
            "outputTokens": 3,
            "cacheReadTokens": 2,
            "cacheWriteTokens": 1
        });
        record_sdk_usage(Some(&t), &usage);
        let (tokens_in, tokens_out, cache_read, cache_write) = {
            let g = t.lock().unwrap();
            (g.tokens_in, g.tokens_out, g.cache_read, g.cache_write)
        };
        assert_eq!(tokens_in, Some(13));
        assert_eq!(tokens_out, Some(3));
        assert_eq!(cache_read, Some(2));
        assert_eq!(cache_write, Some(1));
    }
}
