use serde_json::{Value, json};

use crate::bridge_protocol::{BridgeEvent, RunDoneStatus};

use super::map_event_summary::tool_summary_from_pi;

#[must_use]
pub(crate) fn map_pi_event(type_name: &str, raw: &Value) -> Vec<BridgeEvent> {
    match type_name {
        "message_update" => map_message_update(raw),
        "tool_execution_start" => vec![tool_call_from_execution(raw, "start")],
        "tool_execution_end" => vec![tool_call_from_execution(raw, tool_end_phase(raw))],
        "tool_execution_update" => vec![tool_call_from_execution(raw, "update")],
        "extension_ui_request" | "progress" => map_control_event(type_name, raw),
        "agent_end" => vec![map_agent_end(raw)],
        "text_delta" => text_delta_top_level(raw),
        "thinking_delta" => thinking_delta_top_level(raw),
        _ => Vec::new(),
    }
}

fn map_control_event(type_name: &str, raw: &Value) -> Vec<BridgeEvent> {
    if type_name == "progress" {
        return vec![BridgeEvent::Progress {
            kind: raw.get("kind").and_then(Value::as_str).map(str::to_string),
            detail: raw
                .get("detail")
                .and_then(Value::as_str)
                .map(str::to_string),
        }];
    }
    vec![BridgeEvent::Fatal {
        message: "pi extension_ui_request is unsupported in malvin non-interactive mode".into(),
        retryable: Some(false),
    }]
}

fn tool_end_phase(raw: &Value) -> &'static str {
    if raw.get("isError").and_then(Value::as_bool).unwrap_or(false) {
        "error"
    } else {
        "complete"
    }
}

fn map_message_update(raw: &Value) -> Vec<BridgeEvent> {
    let Some(ame) = raw.get("assistantMessageEvent") else {
        return Vec::new();
    };
    let kind = ame.get("type").and_then(Value::as_str).unwrap_or("");
    match kind {
        "text_delta" => {
            let text = ame
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![BridgeEvent::Assistant { text }]
            }
        }
        "thinking_delta" => {
            let text = ame
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![BridgeEvent::Thinking { text }]
            }
        }
        "toolcall_start" | "toolcall_end" | "toolcall_delta" => Vec::new(),
        _ => Vec::new(),
    }
}

fn tool_call_from_execution(raw: &Value, phase: &str) -> BridgeEvent {
    let tool_call_id = raw
        .get("toolCallId")
        .or_else(|| raw.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let name = raw
        .get("toolName")
        .or_else(|| raw.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let summary = tool_summary_from_pi(name.as_deref(), raw.get("args"));
    BridgeEvent::ToolCall {
        phase: phase.into(),
        name,
        summary,
        tool_call_id,
    }
}

fn map_agent_end(raw: &Value) -> BridgeEvent {
    let result = last_assistant_text(raw);
    let usage = aggregate_usage(raw);
    let err = raw.get("error").and_then(|e| {
        if e.is_null() {
            None
        } else {
            e.as_str()
                .map(str::to_string)
                .or_else(|| Some(e.to_string()))
        }
    });
    let status = if err.is_some() {
        RunDoneStatus::Error
    } else {
        RunDoneStatus::Finished
    };
    BridgeEvent::RunDone {
        status,
        result,
        usage,
        error: err,
        duration_ms: None,
    }
}

fn last_assistant_text(raw: &Value) -> Option<String> {
    let messages = raw.get("messages").and_then(Value::as_array)?;
    for msg in messages.iter().rev() {
        if msg.get("role").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        if let Some(text) = assistant_message_text(msg) {
            return Some(text);
        }
    }
    None
}

fn assistant_message_text(msg: &Value) -> Option<String> {
    if let Some(s) = msg.get("content").and_then(Value::as_str) {
        if !s.is_empty() {
            return Some(s.to_string());
        }
    }
    let parts = msg.get("content").and_then(Value::as_array)?;
    let mut text = String::new();
    for part in parts {
        if part.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        if let Some(t) = part.get("text").and_then(Value::as_str) {
            text.push_str(t);
        }
    }
    (!text.is_empty()).then_some(text)
}

fn aggregate_usage(raw: &Value) -> Option<Value> {
    let messages = raw.get("messages").and_then(Value::as_array)?;
    let mut input = 0_u64;
    let mut output = 0_u64;
    let mut cache_read = 0_u64;
    let mut cache_write = 0_u64;
    let mut seen = false;
    for msg in messages {
        let Some(usage) = msg.get("usage") else {
            continue;
        };
        seen = true;
        input = input.saturating_add(usage_u64(usage, "input"));
        output = output.saturating_add(usage_u64(usage, "output"));
        cache_read = cache_read.saturating_add(usage_u64(usage, "cacheRead"));
        cache_write = cache_write.saturating_add(usage_u64(usage, "cacheWrite"));
    }
    if !seen {
        return None;
    }
    Some(json!({
        "input": input,
        "output": output,
        "cacheRead": cache_read,
        "cacheWrite": cache_write,
    }))
}

fn usage_u64(usage: &Value, key: &str) -> u64 {
    usage
        .get(key)
        .and_then(Value::as_u64)
        .or_else(|| {
            usage
                .get(key)
                .and_then(Value::as_i64)
                .map(|n| n.max(0).cast_unsigned())
        })
        .unwrap_or(0)
}

fn text_delta_top_level(raw: &Value) -> Vec<BridgeEvent> {
    top_level_delta_text(raw).map_or_else(Vec::new, |text| vec![BridgeEvent::Assistant { text }])
}

fn thinking_delta_top_level(raw: &Value) -> Vec<BridgeEvent> {
    top_level_delta_text(raw).map_or_else(Vec::new, |text| vec![BridgeEvent::Thinking { text }])
}

fn top_level_delta_text(raw: &Value) -> Option<String> {
    let text = raw
        .get("delta")
        .or_else(|| raw.get("data"))
        .or_else(|| raw.get("text"))
        .and_then(Value::as_str)
        .unwrap_or("");
    (!text.is_empty()).then(|| text.to_string())
}
