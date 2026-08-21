use serde_json::Value;

use crate::bridge_protocol::BridgeEvent;

use super::map_event_summary::tool_name_summary;

pub(crate) fn map_codex_stream_events(method: &str, params: &Value) -> Vec<BridgeEvent> {
    match method {
        "item/agentMessage/delta" => assistant_delta(params),
        "item/reasoning/textDelta" | "item/reasoning/summaryTextDelta" => thinking_delta(params),
        "item/started" => item_tool_event(params, "start"),
        "item/completed" => item_completed_events(params),
        "thread/tokenUsage/updated" => super::map_event_usage::usage_event(params),
        _ => Vec::new(),
    }
}

fn assistant_delta(params: &Value) -> Vec<BridgeEvent> {
    delta_text(params)
        .map(|text| BridgeEvent::Assistant { text })
        .into_iter()
        .collect()
}

fn thinking_delta(params: &Value) -> Vec<BridgeEvent> {
    delta_text(params)
        .map(|text| BridgeEvent::Thinking { text })
        .into_iter()
        .collect()
}

fn delta_text(params: &Value) -> Option<String> {
    params
        .get("delta")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn item_tool_event(params: &Value, default_phase: &str) -> Vec<BridgeEvent> {
    params
        .get("item")
        .and_then(|item| tool_from_item(item, default_phase))
        .into_iter()
        .collect()
}

fn item_completed_events(params: &Value) -> Vec<BridgeEvent> {
    let Some(item) = params.get("item") else {
        return Vec::new();
    };
    if let Some(ev) = tool_from_item(item, "complete") {
        return vec![ev];
    }
    reasoning_from_item(item).into_iter().collect()
}

fn reasoning_from_item(item: &Value) -> Option<BridgeEvent> {
    if item.get("type").and_then(Value::as_str) != Some("reasoning") {
        return None;
    }
    let text =
        joined_strings(item.get("content")).or_else(|| joined_strings(item.get("summary")))?;
    Some(BridgeEvent::Thinking { text })
}

fn joined_strings(value: Option<&Value>) -> Option<String> {
    let parts: Vec<&str> = value?
        .as_array()?
        .iter()
        .filter_map(string_part)
        .filter(|s| !s.is_empty())
        .collect();
    (!parts.is_empty()).then(|| parts.join(""))
}

fn string_part(value: &Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.get("text").and_then(Value::as_str))
}

fn tool_from_item(item: &Value, default_phase: &str) -> Option<BridgeEvent> {
    let ty = item.get("type").and_then(Value::as_str)?;
    let (name, summary) = tool_name_summary(ty, item)?;
    Some(BridgeEvent::ToolCall {
        phase: tool_phase(default_phase, item.get("status").and_then(Value::as_str)).into(),
        name: Some(name),
        summary: Some(summary),
        tool_call_id: item.get("id").and_then(Value::as_str).map(str::to_string),
    })
}

fn tool_phase(default_phase: &str, status: Option<&str>) -> &'static str {
    if default_phase == "start" {
        return "start";
    }
    match status {
        Some("failed" | "declined") => "error",
        _ => "complete",
    }
}
