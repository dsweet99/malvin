use pi::model::AssistantMessageEvent;
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
            is_error,
            ..
        } => vec![tool_call(
            tool_call_id,
            tool_name,
            &serde_json::Value::Null,
            if *is_error { "error" } else { "complete" },
        )],
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
    }
}
