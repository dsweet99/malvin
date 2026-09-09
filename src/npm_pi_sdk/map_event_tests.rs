use super::map_event::map_npm_pi_event;
use super::session_turn::TurnState;

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
        "toolName": "bash"
    });
    assert_eq!(map_npm_pi_event(&start, &mut state).len(), 1);
    let settled = serde_json::json!({"type": "agent_settled"});
    assert!(map_npm_pi_event(&settled, &mut state).is_empty());
    assert!(state.settled);
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
