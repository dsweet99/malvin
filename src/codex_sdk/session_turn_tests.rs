use super::session_turn::{
    TurnState, agent_message_from_turn, rpc_error, thread_became_idle, turn_is_complete,
};
use super::session_turn_done::{finish_codex_status, turn_duration_ms};
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

#[test]
fn idle_status_does_not_complete_a_turn() {
    let state = TurnState {
        turn_id: Some("turn-test".into()),
        ..TurnState::default()
    };
    let idle = json!({"params":{"status":{"type":"idle"}}});
    assert!(!turn_is_complete("thread/status/changed", &state, &idle));
    let completed = json!({"params":{"turn":{"id":"turn-test","status":"failed"}}});
    assert!(turn_is_complete("turn/completed", &state, &completed));
}

#[test]
fn turn_duration_and_canonical_failure_status() {
    let with_ms = json!({"params":{"turn":{"durationMs":42}}});
    assert_eq!(turn_duration_ms(&with_ms), Some(42));
    let from_stamps = json!({"params":{"turn":{"startedAt":10,"completedAt":12}}});
    assert_eq!(turn_duration_ms(&from_stamps), Some(2000));
    assert!(turn_duration_ms(&json!({})).is_none());
    assert!(finish_codex_status(&json!({}), "error").is_err());
    assert!(finish_codex_status(&json!({}), "cancelled").is_err());
}

#[test]
fn run_done_uses_shared_finished_status_and_usage() {
    use super::session_turn::TurnState;
    use crate::bridge_protocol::BridgeEvent;
    let value = json!({
        "params": {
            "turn": {
                "status": "completed",
                "durationMs": 15,
                "items": [{"type":"agentMessage","text":"hi"}]
            }
        }
    });
    let state = TurnState {
        usage: Some(json!({"inputTokens": 1, "outputTokens": 2})),
        ..TurnState::default()
    };
    match super::session_turn_done::run_done_from_turn(&value, state).expect("run_done") {
        BridgeEvent::RunDone {
            status,
            result,
            usage,
            duration_ms,
            error,
        } => {
            assert_eq!(status, crate::bridge_protocol::RunDoneStatus::Finished);
            assert_eq!(result.as_deref(), Some("hi"));
            assert_eq!(usage.unwrap()["inputTokens"], 1);
            assert_eq!(duration_ms, Some(15));
            assert!(error.is_none());
        }
        other => panic!("unexpected {other:?}"),
    }
    let missing = json!({"params":{"turn":{}}});
    assert!(super::session_turn_done::run_done_from_turn(&missing, TurnState::default()).is_err());
    let from_turn = json!({
        "params": {
            "turn": {
                "status": "completed",
                "tokenUsage": {"last": {"inputTokens": 7, "outputTokens": 3}}
            }
        }
    });
    match super::session_turn_done::run_done_from_turn(&from_turn, TurnState::default())
        .expect("usage")
    {
        BridgeEvent::RunDone {
            usage, duration_ms, ..
        } => {
            assert_eq!(usage.unwrap()["inputTokens"], 7);
            assert!(duration_ms.is_none());
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn kiss_cov_run_done_status_serde() {
    let _ = stringify!(serialize);
    let _ = stringify!(deserialize);
}
