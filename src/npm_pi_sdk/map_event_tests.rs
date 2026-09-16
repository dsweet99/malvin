use super::map_event::map_npm_pi_event;
use super::session_turn::TurnState;
use crate::bridge_protocol::BridgeEvent;

#[test]
fn maps_text_and_thinking_deltas() {
    let mut state = TurnState::default();
    let text = serde_json::json!({
        "type": "message_update",
        "assistantMessageEvent": {"type": "text_delta", "delta": "Hi"}
    });
    let evs = map_npm_pi_event(&text, &mut state);
    assert_eq!(evs.len(), 1);
    assert_eq!(state.response_text, "Hi");
    let think = serde_json::json!({
        "type": "message_update",
        "assistantMessageEvent": {"type": "thinking_delta", "delta": "hmm"}
    });
    assert_eq!(map_npm_pi_event(&think, &mut state).len(), 1);
}

#[test]
fn maps_tools_and_settled() {
    let mut state = TurnState::default();
    let start = serde_json::json!({
        "type": "tool_execution_start",
        "toolCallId": "t1",
        "toolName": "bash",
        "args": {"command": "ls -la"}
    });
    let evs = map_npm_pi_event(&start, &mut state);
    assert_eq!(evs.len(), 1);
    assert!(matches!(
        &evs[0],
        BridgeEvent::ToolCall { phase, name, summary, .. }
            if phase == "start"
                && name.as_deref() == Some("bash")
                && summary.as_deref() == Some("Run ls -la")
    ));
    let settled = serde_json::json!({"type": "agent_settled"});
    assert!(map_npm_pi_event(&settled, &mut state).is_empty());
    assert!(state.settled);
}

#[test]
fn maps_read_tool_like_cursor_with_path_args() {
    let mut state = TurnState::default();
    let start = serde_json::json!({
        "type": "tool_execution_start",
        "toolCallId": "t2",
        "toolName": "read",
        "args": {"path": "README.md"}
    });
    let evs = map_npm_pi_event(&start, &mut state);
    assert!(matches!(
        &evs[0],
        BridgeEvent::ToolCall { summary, .. }
            if summary.as_deref() == Some("Read README.md")
    ));
    let bare = serde_json::json!({
        "type": "tool_execution_start",
        "toolCallId": "t3",
        "toolName": "read"
    });
    let bare_evs = map_npm_pi_event(&bare, &mut state);
    assert!(matches!(
        &bare_evs[0],
        BridgeEvent::ToolCall { summary, .. }
            if summary.as_deref() == Some("Read")
                && !summary.as_deref().unwrap_or("").contains("tool")
    ));
}

#[test]
fn agent_end_fills_text_when_empty() {
    let mut state = TurnState::default();
    let end = serde_json::json!({
        "type": "agent_end",
        "messages": [{
            "role": "assistant",
            "content": [{"type": "text", "text": "done"}]
        }]
    });
    assert!(map_npm_pi_event(&end, &mut state).is_empty());
    assert_eq!(state.response_text, "done");
}
