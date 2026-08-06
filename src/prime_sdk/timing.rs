//! `RunTiming` hooks for SDK bridge events.

use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::run_timing::RunTiming;

pub fn prime_note_sdk_step(timing: Option<&Arc<Mutex<RunTiming>>>) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.steps = g.steps.saturating_add(1);
}

pub fn prime_record_sdk_usage(timing: Option<&Arc<Mutex<RunTiming>>>, usage: &Value) {
    let Some(t) = timing else {
        return;
    };
    let normalized = normalize_prime_usage_to_acp(usage);
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.record_acp_usage_if_present(&normalized);
}

/// Map Prime / pi-ai usage field names onto ACP-style keys expected by [`RunTiming`].
fn normalize_prime_usage_to_acp(usage: &Value) -> Value {
    let Some(obj) = usage.as_object() else {
        return usage.clone();
    };
    let mut out = serde_json::Map::new();
    // Pass through Cursor/ACP names when already present.
    for (k, v) in obj {
        out.insert(k.clone(), v.clone());
    }
    copy_u64_alias(obj, &mut out, "input", "inputTokens");
    copy_u64_alias(obj, &mut out, "output", "outputTokens");
    copy_u64_alias(obj, &mut out, "cacheRead", "cacheReadTokens");
    copy_u64_alias(obj, &mut out, "cacheWrite", "cacheWriteTokens");
    Value::Object(out)
}

fn copy_u64_alias(
    src: &serde_json::Map<String, Value>,
    dest: &mut serde_json::Map<String, Value>,
    from: &str,
    to: &str,
) {
    if dest.contains_key(to) {
        return;
    }
    if let Some(v) = src.get(from) {
        if v.as_u64().is_some() || v.as_i64().is_some() || v.as_f64().is_some() {
            dest.insert(to.to_string(), v.clone());
        }
    }
}
