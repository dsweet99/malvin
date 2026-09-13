use serde_json::Value;

use super::session_turn::TurnState;
use crate::bridge_protocol::BridgeEvent;

pub(super) use super::extension_ui::auto_reply_extension_ui;

pub(super) fn map_npm_pi_event(value: &Value, state: &mut TurnState) -> Vec<BridgeEvent> {
    let ty = value.get("type").and_then(Value::as_str).unwrap_or("");
    match ty {
        "message_update" => map_message_update(value, state),
        "tool_execution_start" => vec![tool_call(value, "start")],
        "tool_execution_update" => vec![tool_call(value, "update")],
        "tool_execution_end" => vec![tool_call(
            value,
            if value.get("isError").and_then(Value::as_bool) == Some(true) {
                "error"
            } else {
                "complete"
            },
        )],
        "agent_end" => {
            capture_agent_end(value, state);
            Vec::new()
        }
        "agent_settled" => {
            state.settled = true;
            Vec::new()
        }
        "extension_error" => vec![BridgeEvent::Fatal {
            message: format!(
                "npm pi extension error: {}",
                value.get("error").unwrap_or(&Value::Null)
            ),
            retryable: Some(false),
        }],
        _ => Vec::new(),
    }
}

fn map_message_update(value: &Value, state: &mut TurnState) -> Vec<BridgeEvent> {
    if let Some(usage) = value.get("usage") {
        state.usage = Some(usage.clone());
    }
    let Some(ev) = value.get("assistantMessageEvent") else {
        return Vec::new();
    };
    let ty = ev.get("type").and_then(Value::as_str).unwrap_or("");
    match ty {
        "text_delta" => delta_text(ev)
            .map(|text| {
                state.response_text.push_str(&text);
                BridgeEvent::Assistant { text }
            })
            .into_iter()
            .collect(),
        "thinking_delta" => delta_text(ev)
            .map(|text| BridgeEvent::Thinking { text })
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn delta_text(ev: &Value) -> Option<String> {
    ev.get("delta")
        .and_then(Value::as_str)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
}

fn capture_agent_end(value: &Value, state: &mut TurnState) {
    if let Some(usage) = value.get("usage") {
        state.usage = Some(usage.clone());
    }
    if !state.response_text.is_empty() {
        return;
    }
    if let Some(text) = last_assistant_from_messages(value.get("messages")) {
        state.response_text = text;
    }
}

fn last_assistant_from_messages(messages: Option<&Value>) -> Option<String> {
    let arr = messages?.as_array()?;
    for msg in arr.iter().rev() {
        if msg.get("role").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        if let Some(text) = collect_text_content(msg.get("content")) {
            return Some(text);
        }
    }
    None
}

fn collect_text_content(content: Option<&Value>) -> Option<String> {
    let arr = content?.as_array()?;
    let mut out = String::new();
    for block in arr {
        if block.get("type").and_then(Value::as_str) == Some("text")
            && let Some(t) = block.get("text").and_then(Value::as_str)
        {
            out.push_str(t);
        }
    }
    (!out.is_empty()).then_some(out)
}

fn tool_call(value: &Value, phase: &str) -> BridgeEvent {
    let name = value
        .get("toolName")
        .and_then(Value::as_str)
        .map(str::to_string);
    let tool_call_id = value
        .get("toolCallId")
        .and_then(Value::as_str)
        .map(str::to_string);
    let summary = name
        .as_deref()
        .map(|n| format!("{n} tool"))
        .or_else(|| Some(format!("tool {phase}")));
    BridgeEvent::ToolCall {
        phase: phase.into(),
        name,
        summary,
        tool_call_id,
    }
}
