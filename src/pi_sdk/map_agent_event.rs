use pi::model::{AssistantMessageEvent, ContentBlock};
use pi::sdk::AgentEvent;

use crate::bridge_protocol::BridgeEvent;

use super::map_agent_event_end::map_agent_end;
use super::map_event_summary::tool_summary_from_pi;

#[must_use]
pub(crate) fn map_pi_agent_event(event: &AgentEvent) -> Vec<BridgeEvent> {
    match event {
        AgentEvent::MessageUpdate {
            assistant_message_event,
            ..
        } => map_assistant_message_event(assistant_message_event),
        AgentEvent::ToolExecutionStart {
            tool_call_id,
            tool_name,
            args,
        } => vec![tool_call(tool_call_id, tool_name, args, "start")],
        AgentEvent::ToolExecutionUpdate {
            tool_call_id,
            tool_name,
            args,
            ..
        } => vec![tool_call(tool_call_id, tool_name, args, "update")],
        AgentEvent::ToolExecutionEnd {
            tool_call_id,
            tool_name,
            result,
            is_error,
        } => {
            let mut ev = tool_call(
                tool_call_id,
                tool_name,
                &serde_json::Value::Null,
                if *is_error { "error" } else { "complete" },
            );
            if *is_error && let BridgeEvent::ToolCall { error, .. } = &mut ev {
                *error = tool_error_text(&result.content);
            }
            vec![ev]
        }
        AgentEvent::AgentEnd {
            messages, error, ..
        } => vec![map_agent_end(messages, error.as_deref())],
        AgentEvent::ExtensionError { error, .. } => vec![BridgeEvent::Fatal {
            message: format!("pi extension event is unsupported: {error}"),
            retryable: Some(false),
        }],
        _ => Vec::new(),
    }
}

fn map_assistant_message_event(event: &AssistantMessageEvent) -> Vec<BridgeEvent> {
    match event {
        AssistantMessageEvent::TextDelta { delta, .. } if !delta.is_empty() => {
            vec![BridgeEvent::Assistant {
                text: delta.clone(),
            }]
        }
        AssistantMessageEvent::ThinkingDelta { delta, .. } if !delta.is_empty() => {
            vec![BridgeEvent::Thinking {
                text: delta.clone(),
            }]
        }
        _ => Vec::new(),
    }
}

fn tool_call(
    tool_call_id: &str,
    tool_name: &str,
    args: &serde_json::Value,
    phase: &str,
) -> BridgeEvent {
    let summary = tool_summary_from_pi(Some(tool_name), Some(args));
    BridgeEvent::ToolCall {
        phase: phase.into(),
        name: Some(tool_name.to_string()),
        summary,
        tool_call_id: Some(tool_call_id.to_string()),
        error: None,
    }
}

const TOOL_ERROR_MAX_CHARS: usize = 500;

fn tool_error_text(content: &[ContentBlock]) -> Option<String> {
    let joined = content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text(part) => Some(part.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let trimmed = joined.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out: String = trimmed.chars().take(TOOL_ERROR_MAX_CHARS).collect();
    if trimmed.chars().nth(TOOL_ERROR_MAX_CHARS).is_some() {
        out.push('…');
    }
    Some(out)
}
