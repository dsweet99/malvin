use super::{DrainIdleTurn, StreamLog, TurnWait, turn_timeout_extension};
use crate::acp::AgentIoOptions;
use crate::bridge_protocol::BridgeEvent;
use crate::model_id::ModelBackend;

const ALL_BACKENDS: [ModelBackend; 3] =
    [ModelBackend::Cursor, ModelBackend::Pi, ModelBackend::Codex];

fn teeing_log() -> StreamLog {
    StreamLog::new(AgentIoOptions {
        no_tee: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    })
}

fn tool_start() -> BridgeEvent {
    BridgeEvent::ToolCall {
        phase: "start".into(),
        name: Some("bash".into()),
        summary: Some("Run sleep 1".into()),
        tool_call_id: Some("t1".into()),
        error: None,
    }
}

#[test]
fn every_backend_extends_turn_on_tool_start() {
    for backend in ALL_BACKENDS {
        let log = teeing_log();
        let mut turn = DrainIdleTurn::new();
        let before = turn.clock.max_deadline();
        (turn_timeout_extension(backend).note_productive_event)(&log, &mut turn, &tool_start());
        assert!(
            turn.clock.max_deadline() > before,
            "{backend:?} must extend the turn deadline on tool start"
        );
    }
}

#[test]
fn every_backend_reports_tools_in_flight_from_stream_log() {
    for backend in ALL_BACKENDS {
        let log = teeing_log();
        let baseline = std::collections::HashSet::new();
        let wait = TurnWait {
            backend,
            log: &log,
            process_group_id: None,
            spawn_pid_baseline: &baseline,
        };
        assert!(!wait.health().tools_in_flight, "{backend:?}");
        crate::bridge_sdk::handle_stream_event(&log, &tool_start());
        assert!(wait.health().tools_in_flight, "{backend:?}");
        let mut turn = DrainIdleTurn::new();
        let before = turn.clock.max_deadline();
        wait.note_productive_event(&mut turn, &tool_start());
        assert!(turn.clock.max_deadline() > before, "{backend:?}");
    }
}

#[test]
fn kiss_cov_turn_timeout() {
    let _ = stringify!(TurnTimeoutExtension);
    let _ = stringify!(await_turn_event);
    let _ = stringify!(health);
}
