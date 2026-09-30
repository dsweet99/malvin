use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::sync::mpsc;

use crate::acp::AgentIoOptions;
use crate::bridge_sdk::StreamLog;

use super::map_agent_event_end::OUTPUT_CAP_MESSAGE;
use super::session::{PiEmbeddedSession, drain_agent_events};
use super::session_fake::{FAKE_OUTPUT_CAP_TOKENS, FAKE_TOOL_ERROR_TEXT, fake_events_for_prompt};

fn minimal_session() -> PiEmbeddedSession {
    capped_session(None)
}

fn capped_session(output_cap: Option<u64>) -> PiEmbeddedSession {
    PiEmbeddedSession {
        runtime: None,
        log: StreamLog::new(AgentIoOptions {
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        }),
        work_dir: std::env::temp_dir(),
        reader_dead: Arc::new(AtomicBool::new(false)),
        spawn_pid_baseline: HashSet::new(),
        pi_provider: String::new(),
        pi_model: String::new(),
        local_hold: false,
        output_cap,
    }
}

#[tokio::test]
async fn agent_end_before_reply_oneshot_returns_ok() {
    let session = minimal_session();
    let (events_tx, events_rx) = mpsc::unbounded_channel();
    for event in fake_events_for_prompt("AGENT_END_BEFORE_ACK", "", "") {
        events_tx.send(event).expect("event");
    }
    drop(events_tx);
    let (_reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    drain_agent_events(&session, events_rx, reply_rx)
        .await
        .expect("run_done success must not wait for reply oneshot");
    assert_eq!(session.log.last_text(), "early-end");
}

async fn drain_fake_prompt(
    session: &PiEmbeddedSession,
    prompt: &str,
) -> Result<(), crate::acp::AgentError> {
    let (events_tx, events_rx) = mpsc::unbounded_channel();
    for event in fake_events_for_prompt(prompt, "", "") {
        events_tx.send(event).expect("event");
    }
    drop(events_tx);
    let (_reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    drain_agent_events(session, events_rx, reply_rx).await
}

#[tokio::test]
async fn empty_turn_at_output_cap_fails_with_output_cap_fault() {
    let session = capped_session(Some(FAKE_OUTPUT_CAP_TOKENS));
    let err = drain_fake_prompt(&session, "OUTPUT_CAP_THINKING")
        .await
        .expect_err("empty turn at the output cap must fail");
    assert_eq!(err.fault, crate::acp::AgentFault::OutputCap);
    assert!(err.message.contains(OUTPUT_CAP_MESSAGE), "{}", err.message);
}

#[tokio::test]
async fn empty_turn_below_cap_or_without_cap_still_succeeds() {
    let above = capped_session(Some(FAKE_OUTPUT_CAP_TOKENS + 1));
    drain_fake_prompt(&above, "OUTPUT_CAP_THINKING")
        .await
        .expect("output below the cap is not a cap failure");
    let uncapped = capped_session(None);
    drain_fake_prompt(&uncapped, "OUTPUT_CAP_THINKING")
        .await
        .expect("sessions without a malvin-written cap are unchecked");
    let texty = capped_session(Some(1));
    drain_fake_prompt(&texty, "hello")
        .await
        .expect("visible text is never a cap failure");
}

fn tool_call_line<'a>(trace: &'a str, phase: &str) -> &'a str {
    let needle = format!("\"phase\":\"{phase}\"");
    trace
        .lines()
        .find(|l| l.contains("tool_call") && l.contains(&needle))
        .unwrap_or_else(|| panic!("no tool_call {phase} line in:\n{trace}"))
}

#[tokio::test]
async fn failed_tool_error_text_reaches_trace_jsonl() {
    let tmp = tempfile::tempdir().expect("tmp");
    let mut session = minimal_session();
    session.log.run_dir = Some(tmp.path().to_path_buf());
    drain_fake_prompt(&session, "TOOL_ERROR_READ")
        .await
        .expect("a failed tool call does not fail the turn");
    let trace = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    let error_line = tool_call_line(&trace, "error");
    assert!(error_line.contains(FAKE_TOOL_ERROR_TEXT), "{error_line}");
    let start_line = tool_call_line(&trace, "start");
    assert!(!start_line.contains("\"error\""), "{start_line}");
}
