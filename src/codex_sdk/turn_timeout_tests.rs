use super::session::CodexSession;
use super::session_io::turn_wait;
use super::session_process::build_codex_session;
use super::session_turn::{TurnState, handle_codex_event};
use crate::acp::AgentIoOptions;
use crate::bridge_sdk::{BridgeSpawnArgs, DrainIdleTurn};
use serde_json::json;

fn cat_codex_session() -> CodexSession {
    let mut child = tokio::process::Command::new("cat")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn cat");
    let stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let model = crate::model_id::parse_model_id("codex:gpt-5").expect("model");
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
        (child, stdin, stdout, None, std::collections::HashSet::new()),
        None,
    )
}

#[tokio::test]
async fn codex_tool_start_extends_turn_and_marks_tools_in_flight() {
    let session = cat_codex_session();
    let mut turn = DrainIdleTurn::new();
    let before = turn.clock.max_deadline();
    let mut state = TurnState {
        turn_id: Some("turn-1".into()),
        ..TurnState::default()
    };
    assert!(!turn_wait(&session).health().tools_in_flight);
    let started = json!({
        "method": "item/started",
        "params": {
            "turnId": "turn-1",
            "item": {"type": "commandExecution", "id": "cmd-1", "command": "sleep 1"}
        }
    });
    assert!(handle_codex_event(&session, &started, &mut state, &mut turn).is_none());
    assert!(
        turn.clock.max_deadline() > before,
        "codex tool start must extend the turn deadline"
    );
    assert!(
        turn_wait(&session).health().tools_in_flight,
        "codex open tool must report tools_in_flight"
    );
}

#[tokio::test]
async fn codex_non_tool_event_does_not_extend_turn() {
    let session = cat_codex_session();
    let mut turn = DrainIdleTurn::new();
    let before = turn.clock.max_deadline();
    let mut state = TurnState {
        turn_id: Some("turn-1".into()),
        ..TurnState::default()
    };
    let delta = json!({
        "method": "item/agentMessage/delta",
        "params": {"turnId": "turn-1", "delta": "hi"}
    });
    assert!(handle_codex_event(&session, &delta, &mut state, &mut turn).is_none());
    assert_eq!(turn.clock.max_deadline(), before);
    assert!(!turn_wait(&session).health().tools_in_flight);
}
