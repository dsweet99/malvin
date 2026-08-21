use serde_json::json;

use super::map_event::map_pi_event;
use crate::bridge_protocol::BridgeEvent;

#[test]
fn maps_text_delta_and_ignores_message_toolcall() {
    let text = map_pi_event(
        "message_update",
        &json!({
            "assistantMessageEvent": { "type": "text_delta", "delta": "hi" }
        }),
    );
    assert!(matches!(
        text.as_slice(),
        [BridgeEvent::Assistant { text }] if text == "hi"
    ));

    let tool = map_pi_event(
        "message_update",
        &json!({
            "assistantMessageEvent": {
                "type": "toolcall_start",
                "partial": {
                    "content": [{ "type": "toolCall", "id": "c1", "name": "ls" }]
                }
            }
        }),
    );
    assert!(tool.is_empty(), "unexpected {tool:?}");
}

#[test]
fn maps_agent_end_to_run_done_with_usage() {
    let evs = map_pi_event(
        "agent_end",
        &json!({
            "error": null,
            "messages": [
                { "role": "user", "content": "hi" },
                {
                    "role": "assistant",
                    "content": [{ "type": "text", "text": "yo" }],
                    "usage": { "input": 10, "output": 2, "cacheRead": 0, "cacheWrite": 0 }
                }
            ]
        }),
    );
    match evs.as_slice() {
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
                    .and_then(|u| u.get("input"))
                    .and_then(serde_json::Value::as_u64),
                Some(10)
            );
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn extension_ui_request_is_fatal() {
    let evs = map_pi_event("extension_ui_request", &json!({}));
    assert!(matches!(
        evs.as_slice(),
        [BridgeEvent::Fatal {
            retryable: Some(false),
            ..
        }]
    ));
}

#[test]
fn maps_progress_for_protocol_compatibility() {
    let events = map_pi_event(
        "progress",
        &json!({ "kind": "heartbeat", "detail": "waiting" }),
    );
    assert!(matches!(
        events.as_slice(),
        [BridgeEvent::Progress { kind, detail }]
            if kind.as_deref() == Some("heartbeat")
                && detail.as_deref() == Some("waiting")
    ));
}

#[test]
fn maps_tool_execution_and_top_level_deltas() {
    let start = map_pi_event(
        "tool_execution_start",
        &json!({
            "toolCallId": "t1",
            "toolName": "bash",
            "args": { "command": "ls -la" }
        }),
    );
    assert!(matches!(
        start.as_slice(),
        [BridgeEvent::ToolCall { phase, name, summary, .. }]
            if phase == "start"
                && name.as_deref() == Some("bash")
                && summary.as_deref() == Some("Run ls -la")
    ));
    let end = map_pi_event(
        "tool_execution_end",
        &json!({
            "toolCallId": "t1",
            "toolName": "bash",
            "args": { "command": "ls -la" },
            "isError": false
        }),
    );
    assert!(matches!(
        end.as_slice(),
        [BridgeEvent::ToolCall { phase, .. }] if phase == "complete"
    ));
    let err = map_pi_event(
        "tool_execution_end",
        &json!({
            "toolCallId": "t2",
            "toolName": "bash",
            "isError": true
        }),
    );
    assert!(matches!(
        err.as_slice(),
        [BridgeEvent::ToolCall { phase, .. }] if phase == "error"
    ));
    let think = map_pi_event("thinking_delta", &json!({ "delta": "hmm" }));
    assert!(matches!(
        think.as_slice(),
        [BridgeEvent::Thinking { text }] if text == "hmm"
    ));
    let text = map_pi_event("text_delta", &json!({ "text": "ok" }));
    assert!(matches!(
        text.as_slice(),
        [BridgeEvent::Assistant { text }] if text == "ok"
    ));
}
#[test]
fn maps_agent_end_error_and_plain_content() {
    let evs = map_pi_event(
        "agent_end",
        &json!({
            "error": "boom",
            "messages": [{ "role": "assistant", "content": "plain" }]
        }),
    );
    match evs.as_slice() {
        [
            BridgeEvent::RunDone {
                status,
                result,
                error,
                ..
            },
        ] => {
            assert_eq!(*status, crate::bridge_protocol::RunDoneStatus::Error);
            assert_eq!(result.as_deref(), Some("plain"));
            assert_eq!(error.as_deref(), Some("boom"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn covers_tool_end_phase_with_non_bool_is_error() {
    let ev = map_pi_event(
        "tool_execution_end",
        &json!({ "toolCallId": "t3", "toolName": "bash", "isError": "nope" }),
    );
    assert!(matches!(ev.as_slice(), [BridgeEvent::ToolCall { phase, .. }] if phase == "complete"));
    let missing = map_pi_event(
        "tool_execution_end",
        &json!({ "toolCallId": "t4", "toolName": "bash" }),
    );
    assert!(
        matches!(missing.as_slice(), [BridgeEvent::ToolCall { phase, .. }] if phase == "complete")
    );
}

#[test]
fn covers_flatten_ws_via_bash_and_read_summaries() {
    let start = map_pi_event(
        "tool_execution_start",
        &json!({
            "toolCallId": "t5",
            "toolName": "bash",
            "args": { "command": "  ls   -la  \n  " }
        }),
    );
    assert!(matches!(
        start.as_slice(),
        [BridgeEvent::ToolCall { summary: Some(s), .. }] if s == "Run ls -la"
    ));
    let read = map_pi_event(
        "tool_execution_start",
        &json!({
            "toolCallId": "t6",
            "toolName": "read",
            "args": { "path": "  /tmp/a  b.txt  " }
        }),
    );
    assert!(matches!(
        read.as_slice(),
        [BridgeEvent::ToolCall { summary: Some(s), .. }] if s.contains("Read")
    ));
}
