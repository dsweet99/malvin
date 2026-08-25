use pi::model::{
    AssistantMessage, AssistantMessageEvent, ContentBlock, Message, TextContent, Usage,
};
use pi::sdk::AgentEvent;

use super::map_agent_event::map_pi_agent_event;
use crate::bridge_protocol::BridgeEvent;

#[test]
fn maps_typed_text_and_thinking_deltas() {
    let assistant = AssistantMessage {
        content: vec![ContentBlock::Text(TextContent::new("hi"))],
        ..AssistantMessage::default()
    };
    let text = map_pi_agent_event(&AgentEvent::MessageUpdate {
        message: Message::assistant(assistant.clone()),
        assistant_message_event: AssistantMessageEvent::TextDelta {
            content_index: 0,
            delta: "hi".into(),
            partial: std::sync::Arc::new(assistant.clone()),
        },
    });
    assert!(matches!(
        text.as_slice(),
        [BridgeEvent::Assistant { text }] if text == "hi"
    ));
    let think = map_pi_agent_event(&AgentEvent::MessageUpdate {
        message: Message::assistant(assistant.clone()),
        assistant_message_event: AssistantMessageEvent::ThinkingDelta {
            content_index: 0,
            delta: "hmm".into(),
            partial: std::sync::Arc::new(assistant),
        },
    });
    assert!(matches!(
        think.as_slice(),
        [BridgeEvent::Thinking { text }] if text == "hmm"
    ));
}

#[test]
fn maps_typed_tool_and_agent_end() {
    let start = map_pi_agent_event(&AgentEvent::ToolExecutionStart {
        tool_call_id: "t1".into(),
        tool_name: "bash".into(),
        args: serde_json::json!({ "command": "ls -la" }),
    });
    assert!(matches!(
        start.as_slice(),
        [BridgeEvent::ToolCall { phase, name, summary, .. }]
            if phase == "start"
                && name.as_deref() == Some("bash")
                && summary.as_deref() == Some("Run ls -la")
    ));
    let end = map_pi_agent_event(&AgentEvent::ToolExecutionEnd {
        tool_call_id: "t1".into(),
        tool_name: "bash".into(),
        result: pi::sdk::ToolOutput {
            content: Vec::new(),
            details: None,
            is_error: false,
        },
        is_error: false,
    });
    assert!(matches!(
        end.as_slice(),
        [BridgeEvent::ToolCall { phase, .. }] if phase == "complete"
    ));
    let assistant = AssistantMessage {
        content: vec![ContentBlock::Text(TextContent::new("yo"))],
        usage: Usage {
            input: 10,
            output: 2,
            cost: pi::model::Cost {
                input: 0.01,
                output: 0.02,
                total: 0.03,
                ..pi::model::Cost::default()
            },
            ..Usage::default()
        },
        ..AssistantMessage::default()
    };
    let done = map_pi_agent_event(&AgentEvent::AgentEnd {
        session_id: "s".into(),
        messages: vec![
            Message::User(pi::model::UserMessage {
                content: pi::model::UserContent::Text("hi".into()),
                timestamp: 0,
            }),
            Message::assistant(assistant),
        ],
        error: None,
    });
    match done.as_slice() {
        [
            BridgeEvent::RunDone {
                status,
                result,
                usage,
                error,
                ..
            },
        ] => {
            assert_eq!(*status, crate::bridge_protocol::RunDoneStatus::Finished);
            assert_eq!(result.as_deref(), Some("yo"));
            assert!(error.is_none());
            assert_eq!(
                usage
                    .as_ref()
                    .and_then(|u| u.get("inputTokens"))
                    .and_then(serde_json::Value::as_u64),
                Some(10)
            );
            assert_eq!(
                usage
                    .as_ref()
                    .and_then(|u| u.get("costUsd"))
                    .and_then(|c| c.get("total"))
                    .and_then(serde_json::Value::as_f64),
                Some(0.03)
            );
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn typed_extension_error_is_fatal() {
    let evs = map_pi_agent_event(&AgentEvent::ExtensionError {
        extension_id: None,
        event: "ui".into(),
        error: "nope".into(),
    });
    assert!(matches!(
        evs.as_slice(),
        [BridgeEvent::Fatal {
            retryable: Some(false),
            ..
        }]
    ));
}
