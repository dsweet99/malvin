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
        "turn_end" => vec![BridgeEvent::Step {
            kind: Some("turn".into()),
        }],
        "message_end" | "agent_end" | "agent_settled" => {
            capture_turn_state(ty, value, state);
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

fn capture_turn_state(ty: &str, value: &Value, state: &mut TurnState) {
    match ty {
        "message_end" => add_assistant_usage(value, state),
        "agent_end" => capture_agent_end(value, state),
        _ => state.settled = true,
    }
}

fn add_assistant_usage(value: &Value, state: &mut TurnState) {
    let Some(message) = value.get("message") else {
        return;
    };
    if message.get("role").and_then(Value::as_str) != Some("assistant") {
        return;
    }
    let Some(usage) = message.get("usage").filter(|u| u.is_object()) else {
        return;
    };
    match state.usage.as_mut() {
        Some(total) => sum_usage(total, usage),
        None => state.usage = Some(usage.clone()),
    }
}

fn sum_usage(total: &mut Value, add: &Value) {
    let (Some(total), Some(add)) = (total.as_object_mut(), add.as_object()) else {
        return;
    };
    for (key, value) in add {
        match total.get_mut(key) {
            Some(slot) if slot.is_object() => sum_usage(slot, value),
            Some(slot) => {
                if let (Some(a), Some(b)) = (slot.as_u64(), value.as_u64()) {
                    *slot = Value::from(a + b);
                } else if let (Some(a), Some(b)) = (slot.as_f64(), value.as_f64()) {
                    *slot = Value::from(a + b);
                }
            }
            None => {
                total.insert(key.clone(), value.clone());
            }
        }
    }
}

fn map_message_update(value: &Value, state: &mut TurnState) -> Vec<BridgeEvent> {
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
    state.end_error =
        super::agent_end_error::agent_end_error(value.get("messages"), state.output_cap);
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
    let args = value.get("args").or_else(|| value.get("arguments"));
    let summary = crate::pi_sdk::tool_summary_from_pi(name.as_deref(), args)
        .or_else(|| Some(format!("tool {phase}")));
    BridgeEvent::ToolCall {
        phase: phase.into(),
        name,
        summary,
        tool_call_id,
        error: None,
    }
}
