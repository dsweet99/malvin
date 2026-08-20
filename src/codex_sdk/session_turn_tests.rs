use super::session_turn::{
    agent_message_from_turn, finish_codex_status, rpc_error, thread_became_idle,
};
use serde_json::json;

#[test]
fn failed_and_interrupted_turns_are_errors() {
    let failed = json!({"params":{"turn":{"status":"failed","error":{"message":"auth"}}}});
    let err = finish_codex_status(&failed, "failed").unwrap_err();
    assert!(err.0.contains("auth"));
    assert!(finish_codex_status(&json!({}), "interrupted").is_err());
    assert!(finish_codex_status(&json!({}), "completed").is_ok());
    assert!(rpc_error(&json!({"error":{"message":"bad"}})).is_some());
    assert!(rpc_error(&json!({"method":"error"})).is_none());
}

#[test]
fn agent_text_comes_from_items_not_last_agent_message() {
    let value = json!({
        "params": {
            "turn": {
                "items": [
                    {"type":"userMessage","text":"hi"},
                    {"type":"agentMessage","text":"pong."}
                ],
                "lastAgentMessage": "ignored"
            }
        }
    });
    assert_eq!(
        agent_message_from_turn(&value, String::new()).as_deref(),
        Some("pong.")
    );
    assert_eq!(
        agent_message_from_turn(&json!({}), "delta".into()).as_deref(),
        Some("delta")
    );
    assert!(thread_became_idle(
        "thread/status/changed",
        &json!({"params":{"status":{"type":"idle"}}})
    ));
    assert!(!thread_became_idle(
        "thread/status/changed",
        &json!({"params":{"status":{"type":"active"}}})
    ));
}
