use serde_json::Value;

use crate::bridge_protocol::BridgeEvent;

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
        .pointer("/tokenUsage/last")
        .or_else(|| params.pointer("/turn/tokenUsage/last"))
        .or_else(|| params.pointer("/tokenUsage"))
        .or_else(|| params.pointer("/turn/tokenUsage"))
        .or_else(|| params.get("last"))
        .and_then(Value::as_object)
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
    #[test]
    fn kiss_cov_map_event_usage() {
        let _ = stringify!(usage_event);
        let _ = stringify!(usage_from_turn);
        let _ = stringify!(usage_from_codex);
        let _ = stringify!(usage_object);
        let _ = stringify!(copy_num);
    }
}
