use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::sync::mpsc;

use crate::acp::AgentIoOptions;
use crate::bridge_sdk::StreamLog;

use super::session::{PiEmbeddedSession, drain_agent_events};
use super::session_fake::fake_events_for_prompt;

fn minimal_session() -> PiEmbeddedSession {
    PiEmbeddedSession {
        runtime: None,
        log: StreamLog::new(AgentIoOptions {
            force: true,
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
    // Leave reply pending — mirrors real PiRuntime where AgentEnd events
    // arrive before prompt_with_abort sends the ack oneshot.
    let (_reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    drain_agent_events(&session, events_rx, reply_rx)
        .await
        .expect("run_done success must not wait for reply oneshot");
    assert_eq!(session.log.last_text(), "early-end");
}
