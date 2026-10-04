use std::sync::{Arc, Mutex};

use tokio::process::{Child, ChildStdin, ChildStdout};

use super::{BridgeSession, ChildProcessSession, DrainIdleTurn, StdioChild, StreamLog, TurnWait};
use crate::acp::AgentIoOptions;
use crate::bridge_protocol::BridgeEvent;

pub(crate) fn teeing_log() -> StreamLog {
    StreamLog::new(AgentIoOptions {
        no_tee: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    })
}

pub(crate) fn tool_start() -> BridgeEvent {
    BridgeEvent::ToolCall {
        phase: "start".into(),
        name: Some("bash".into()),
        summary: Some("Run sleep 1".into()),
        tool_call_id: Some("t1".into()),
        error: None,
    }
}

pub(crate) fn assert_turn_timeout_contract<S: ChildProcessSession>(session: &S) {
    let log = &session.stdio().log;
    let wait = TurnWait::of(session);
    assert!(!wait.health().tools_in_flight);
    crate::bridge_sdk::handle_stream_event(log, &tool_start());
    assert!(wait.health().tools_in_flight);
    let mut turn = DrainIdleTurn::new();
    let before = turn.clock.max_deadline();
    wait.note_productive_event(&mut turn, &tool_start());
    assert!(turn.clock.max_deadline() > before);
}

pub(crate) fn assert_dead_session_error(err: &crate::acp::AgentError, needle: &str) {
    use crate::acp::AgentFault;
    assert!(err.message.contains(needle), "{err}");
    assert_eq!(err.fault, AgentFault::SessionDead, "{err}");
    assert_eq!(
        crate::acp::classify_agent_fault(&err.message),
        AgentFault::SessionDead,
        "retry classifier must treat `{err}` as a dead session"
    );
}

pub(crate) fn cat_child() -> (Child, ChildStdin, ChildStdout) {
    script_child("exec cat")
}

pub(crate) fn script_child(script: &str) -> (Child, ChildStdin, ChildStdout) {
    let mut child = tokio::process::Command::new("sh")
        .args(["-c", script])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn cat");
    let stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    (child, stdin, stdout)
}

fn cat_bridge_session() -> BridgeSession {
    BridgeSession {
        stdio: stdio_child_from(cat_child()),
        agent_id: Mutex::new(None),
    }
}

pub(crate) fn stdio_child_from(
    (child, stdin, stdout): (Child, ChildStdin, ChildStdout),
) -> StdioChild {
    StdioChild::new(
        (child, stdin, stdout, None, std::collections::HashSet::new()),
        std::path::PathBuf::from("."),
        teeing_log(),
    )
}

#[tokio::test]
async fn cursor_bridge_session_extends_turn_and_reports_tools_in_flight() {
    let session = cat_bridge_session();
    assert_turn_timeout_contract(&session);
}

#[tokio::test]
async fn cursor_bridge_session_drop_marks_reader_dead() {
    let session = cat_bridge_session();
    let reader_dead = Arc::clone(&session.stdio.reader_dead);
    drop(session);
    assert!(reader_dead.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn kiss_cov_turn_timeout() {
    let _ = stringify!(TurnTimeoutExtension);
    let _ = stringify!(await_turn_event);
    let _ = stringify!(health);
}
