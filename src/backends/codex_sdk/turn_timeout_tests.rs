use super::session::CodexSession;
use super::session_process::build_codex_session;
use super::session_turn::{TurnState, handle_codex_event};
use crate::agent_process::AgentIoOptions;
use crate::backends::bridge_sdk::turn_timeout_tests::{assert_dead_session_error, cat_child, script_child};
use crate::backends::bridge_sdk::{BridgeSpawnArgs, DrainIdleTurn, JsonLineSession, TurnWait};
use serde_json::json;

fn cat_codex_session() -> CodexSession {
    codex_session_from(cat_child())
}

fn codex_session_from(
    (child, stdin, stdout): (
        tokio::process::Child,
        tokio::process::ChildStdin,
        tokio::process::ChildStdout,
    ),
) -> CodexSession {
    let model = crate::config::model_id::parse_model_id("codex:gpt-5").expect("model");
    let args = BridgeSpawnArgs {
        cwd: std::path::Path::new("."),
        model: &model,
        thinking: None,
        io: AgentIoOptions {
            no_tee: false,
            raw_output: false,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
        run_dir: None,
        timing: None,
    };
    build_codex_session(
        &args,
        crate::backends::bridge_sdk::SpawnedStdio {
            child,
            stdin,
            stdout,
            pgid: None,
            baseline_pids: std::collections::HashSet::new(),
        },
        None,
    )
}

#[tokio::test]
async fn codex_tool_start_marks_tools_in_flight() {
    let session = cat_codex_session();
    let mut state = TurnState {
        turn_id: Some("turn-1".into()),
        ..TurnState::default()
    };
    assert!(!TurnWait::of(&session).health().tools_in_flight);
    let started = json!({
        "method": "item/started",
        "params": {
            "turnId": "turn-1",
            "item": {"type": "commandExecution", "id": "cmd-1", "command": "sleep 1"}
        }
    });
    assert!(handle_codex_event(&session, &started, &mut state).is_none());
    assert!(
        TurnWait::of(&session).health().tools_in_flight,
        "codex open tool must report tools_in_flight"
    );
}

#[tokio::test]
async fn codex_non_tool_event_leaves_tools_idle() {
    let session = cat_codex_session();
    let mut state = TurnState {
        turn_id: Some("turn-1".into()),
        ..TurnState::default()
    };
    let delta = json!({
        "method": "item/agentMessage/delta",
        "params": {"turnId": "turn-1", "delta": "hi"}
    });
    assert!(handle_codex_event(&session, &delta, &mut state).is_none());
    assert!(!TurnWait::of(&session).health().tools_in_flight);
}

#[tokio::test]
async fn codex_json_line_round_trips_through_child() {
    let session = cat_codex_session();
    let sent = json!({"method": "ping", "id": 7});
    session.write_json(&sent).await.expect("write");
    let mut turn = DrainIdleTurn::new();
    let got = session
        .read_json_waiting("echo", &mut turn)
        .await
        .expect("read");
    assert_eq!(got, sent);
}

#[tokio::test]
async fn codex_json_line_accepts_crlf() {
    let session = codex_session_from(script_child("printf '{\"a\":1}\\r\\n'; exec cat"));
    let mut turn = DrainIdleTurn::new();
    let got = session
        .read_json_waiting("crlf", &mut turn)
        .await
        .expect("read");
    assert_eq!(got, json!({"a": 1}));
}

#[tokio::test]
async fn codex_json_line_failures_are_dead_session_errors() {
    let closed = codex_session_from(script_child("exit 0"));
    let mut turn = DrainIdleTurn::new();
    let err = closed
        .read_json_waiting("eof", &mut turn)
        .await
        .unwrap_err();
    assert_dead_session_error(&err, "codex stdout closed");

    let garbled = codex_session_from(script_child("echo nope; exec cat"));
    let err = garbled
        .read_json_waiting("parse", &mut turn)
        .await
        .unwrap_err();
    assert_dead_session_error(&err, "codex JSON-RPC parse:");
}

#[tokio::test]
async fn codex_session_drop_marks_reader_dead() {
    let session = cat_codex_session();
    let reader_dead = std::sync::Arc::clone(&session.stdio.reader_dead);
    drop(session);
    assert!(reader_dead.load(std::sync::atomic::Ordering::SeqCst));
}
