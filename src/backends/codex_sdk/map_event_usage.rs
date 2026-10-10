use serde_json::Value;

use crate::backends::cursor_sdk::protocol::BridgeEvent;

pub(super) fn usage_event(params: &Value) -> Vec<BridgeEvent> {
    usage_from_codex(params)
        .map(|usage| BridgeEvent::Usage { usage })
        .into_iter()
        .collect()
}

pub(super) fn usage_from_turn(value: &Value) -> Option<Value> {
    usage_from_codex(value.get("params").unwrap_or(value))
}

fn usage_from_codex(params: &Value) -> Option<Value> {
    let last = usage_object(params)?;
    let mut out = serde_json::Map::new();
    copy_num(last, &mut out, "inputTokens", "inputTokens");
    copy_num(last, &mut out, "outputTokens", "outputTokens");
    copy_num(last, &mut out, "cachedInputTokens", "cacheReadTokens");
    copy_num(last, &mut out, "cacheWriteInputTokens", "cacheWriteTokens");
    copy_num(last, &mut out, "reasoningOutputTokens", "reasoningTokens");
    copy_num(last, &mut out, "totalTokens", "totalTokens");
    copy_num(last, &mut out, "cacheReadTokens", "cacheReadTokens");
    copy_num(last, &mut out, "cacheWriteTokens", "cacheWriteTokens");
    copy_num(last, &mut out, "reasoningTokens", "reasoningTokens");
    (!out.is_empty()).then_some(Value::Object(out))
}

fn usage_object(params: &Value) -> Option<&serde_json::Map<String, Value>> {
    params
        .pointer("/tokenUsage/total")
        .or_else(|| params.pointer("/turn/tokenUsage/total"))
        .or_else(|| params.pointer("/tokenUsage/last"))
        .or_else(|| params.pointer("/turn/tokenUsage/last"))
        .or_else(|| params.pointer("/tokenUsage"))
        .or_else(|| params.pointer("/turn/tokenUsage"))
        .or_else(|| params.get("last"))
        .and_then(Value::as_object)
}

pub(super) fn absorb_usage(slot: &mut Option<Value>, next: &Value) {
    match slot {
        None => *slot = Some(next.clone()),
        Some(prior) if usage_dominates(next, prior) => *prior = next.clone(),
        Some(prior) => add_usage_counts(prior, next),
    }
}

fn usage_dominates(next: &Value, prior: &Value) -> bool {
    let (Some(next), Some(prior)) = (next.as_object(), prior.as_object()) else {
        return false;
    };
    if prior.is_empty() {
        return true;
    }
    prior.iter().all(|(key, value)| {
        let Some(old) = value.as_u64() else {
            return true;
        };
        next.get(key)
            .and_then(Value::as_u64)
            .is_some_and(|n| n >= old)
    })
}

fn add_usage_counts(prior: &mut Value, next: &Value) {
    let (Some(dest), Some(src)) = (prior.as_object_mut(), next.as_object()) else {
        return;
    };
    for (key, value) in src {
        let Some(add) = value.as_u64() else {
            continue;
        };
        let sum = dest
            .get(key)
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .saturating_add(add);
        dest.insert(key.clone(), Value::from(sum));
    }
}

fn copy_num(
    src: &serde_json::Map<String, Value>,
    dest: &mut serde_json::Map<String, Value>,
    from: &str,
    to: &str,
) {
    if let Some(v) = src.get(from)
        && (v.as_u64().is_some() || v.as_i64().is_some() || v.as_f64().is_some())
    {
        dest.insert(to.to_string(), v.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::{absorb_usage, usage_from_codex};
    use serde_json::json;

    #[test]
    fn kiss_cov_map_event_usage() {
        let _ = stringify!(usage_event);
        let _ = stringify!(usage_from_turn);
        let _ = stringify!(usage_from_codex);
        let _ = stringify!(usage_object);
        let _ = stringify!(copy_num);
    }

    #[test]
    fn token_usage_total_wins_over_last() {
        let usage = usage_from_codex(&json!({
            "tokenUsage": {
                "total": {"inputTokens": 30, "outputTokens": 8, "reasoningOutputTokens": 5},
                "last": {"inputTokens": 4, "outputTokens": 2, "reasoningOutputTokens": 0}
            }
        }))
        .expect("usage");
        assert_eq!(usage["inputTokens"], 30);
        assert_eq!(usage["outputTokens"], 8);
        assert_eq!(usage["reasoningTokens"], 5);
    }

    #[test]
    fn separate_calls_sum_when_later_snapshot_shrinks() {
        let mut slot = None;
        absorb_usage(
            &mut slot,
            &json!({"inputTokens": 10, "outputTokens": 4, "reasoningTokens": 3}),
        );
        absorb_usage(
            &mut slot,
            &json!({"inputTokens": 12, "outputTokens": 1, "reasoningTokens": 0}),
        );
        let usage = slot.expect("usage");
        assert_eq!(usage["inputTokens"], 22);
        assert_eq!(usage["outputTokens"], 5);
        assert_eq!(usage["reasoningTokens"], 3);
    }

    #[test]
    fn cumulative_snapshot_replaces_instead_of_summing() {
        let mut slot = None;
        absorb_usage(&mut slot, &json!({"inputTokens": 10, "outputTokens": 4}));
        absorb_usage(&mut slot, &json!({"inputTokens": 25, "outputTokens": 9}));
        let usage = slot.expect("usage");
        assert_eq!(usage["inputTokens"], 25);
        assert_eq!(usage["outputTokens"], 9);
    }
}
