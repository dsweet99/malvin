use crate::agent_phase::{heartbeat_label, reset_for_run};
use crate::backends::bridge_sdk::{StreamLog, handle_stream_event};
use crate::backends::cursor_sdk::protocol::BridgeEvent;
use crate::tool_summary::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use serde_json::json;

fn log(no_tee: bool) -> StreamLog {
    StreamLog::new(crate::agent_process::AgentIoOptions {
        no_tee,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    })
}

fn sdk_tool(name: &str, phase: &str, summary: &str) -> BridgeEvent {
    BridgeEvent::ToolCall {
        name: Some(name.into()),
        phase: phase.into(),
        summary: Some(summary.into()),
        tool_call_id: Some("qa-tool".into()),
        error: None,
    }
}

fn legacy(tracker: &mut ToolSummaryTracker, id: &str, kind: &str, status: &str) {
    let session_update = if status == "pending" {
        "tool_call"
    } else {
        "tool_call_update"
    };
    tool_summary_lines(
        &json!({"method":"session/update", "params":{"update":{
            "sessionUpdate":session_update, "toolCallId":id, "kind":kind, "status":status,
            "rawInput":{"command":"echo qa"}
        }}}),
        tracker,
        ToolSummaryDetail::Log,
    )
    .unwrap();
}

#[test]
fn status_sdk_activity_matrix() {
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut failures = Vec::new();
    for no_tee in [true, false] {
        for (name, expected) in [
            ("read", "Reading"),
            ("edit", "Editing"),
            ("shell", "Running"),
            ("search", "Reading"),
        ] {
            reset_for_run();
            let session = log(no_tee);
            handle_stream_event(&session, &sdk_tool(name, "start", "QA tool"));
            let actual = heartbeat_label();
            println!("SDK {name} start no_tee={no_tee}: expected={expected}, actual={actual}");
            if actual != expected {
                failures.push(format!("{name}/{no_tee}: {actual}"));
            }
            handle_stream_event(&session, &sdk_tool(name, "complete", "QA tool"));
            let actual = heartbeat_label();
            println!("SDK {name} complete: expected=Thinking, actual={actual}");
            if actual != "Thinking" {
                failures.push(format!("{name} complete: {actual}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
}

#[test]
fn status_sdk_text_exits_starting() {
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let session = log(true);
    let mut actual = Vec::new();
    for ev in [
        BridgeEvent::Thinking {
            text: "Reasoning".into(),
        },
        BridgeEvent::Assistant {
            text: "Working".into(),
        },
    ] {
        reset_for_run();
        handle_stream_event(&session, &ev);
        actual.push(heartbeat_label());
    }
    assert_eq!(actual, vec!["Thinking", "Thinking"]);
}

#[test]
fn status_codex_mapping_to_heartbeat() {
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_for_run();
    let session = log(true);
    let events = crate::backends::codex_sdk_map_event::map_codex_stream_events(
        "item/started",
        &json!({
            "item":{"id":"qa-codex", "type":"commandExecution", "command":"sleep 60", "status":"inProgress"}
        }),
    );
    assert_eq!(events.len(), 1, "mapping must produce a real tool event");
    for event in events {
        handle_stream_event(&session, &event);
    }
    assert_eq!(heartbeat_label(), "Running");
}

#[test]
fn status_legacy_duplicate_execute_start_is_idempotent() {
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_for_run();
    let mut tracker = ToolSummaryTracker::default();
    legacy(&mut tracker, "a", "execute", "pending");
    legacy(&mut tracker, "a", "execute", "pending");
    legacy(&mut tracker, "a", "execute", "completed");
    assert_eq!(heartbeat_label(), "Thinking");
}

#[test]
fn status_legacy_overlapping_execute_completion_is_idempotent() {
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_for_run();
    let mut tracker = ToolSummaryTracker::default();
    legacy(&mut tracker, "a", "execute", "pending");
    legacy(&mut tracker, "b", "execute", "pending");
    legacy(&mut tracker, "a", "execute", "completed");
    legacy(&mut tracker, "a", "execute", "completed");
    assert_eq!(heartbeat_label(), "Running", "b is still active");
}

#[test]
fn status_sdk_overlap_and_terminal_cleanup() {
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK.lock().unwrap();
    reset_for_run();
    let session = log(true);
    handle_stream_event(&session, &sdk_tool("shell", "start", "Run sleep 1"));
    handle_stream_event(&session, &sdk_tool("shell", "start", "Run sleep 1"));
    let read = BridgeEvent::ToolCall {
        name: Some("read".into()),
        phase: "start".into(),
        summary: None,
        tool_call_id: Some("other".into()),
        error: None,
    };
    handle_stream_event(&session, &read);
    assert_eq!(heartbeat_label(), "Running");
    for _ in 0..2 {
        handle_stream_event(&session, &sdk_tool("", "error", ""));
    }
    assert_eq!(heartbeat_label(), "Reading");
    handle_stream_event(
        &session,
        &BridgeEvent::Fatal {
            message: "lost connection".into(),
            retryable: None,
        },
    );
    assert_eq!(heartbeat_label(), "Thinking");
    handle_stream_event(&session, &read);
    handle_stream_event(
        &session,
        &BridgeEvent::RunDone {
            status: crate::backends::cursor_sdk::protocol::RunDoneStatus::Finished,
            result: None,
            usage: None,
            error: None,
            duration_ms: None,
        },
    );
    assert_eq!(heartbeat_label(), "Thinking");
}

#[test]
fn status_sdk_unknown_tool_and_empty_text_policy() {
    let mut state = super::PhaseState::fresh();
    super::observe_event(
        &mut state,
        &BridgeEvent::Assistant {
            text: String::new(),
        },
    );
    assert_eq!(state.resolve().label(), "Starting");
    super::observe_event(&mut state, &sdk_tool("custom_mcp_tool", "start", "Custom"));
    assert_eq!(state.resolve().label(), "Thinking");
    super::observe_event(&mut state, &sdk_tool("shell", "queued", "Run true"));
    assert_eq!(state.resolve().label(), "Thinking");
}

#[test]
fn status_sdk_failed_check_and_successful_retry() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        std::fs::create_dir_all(work.join(".malvin")).unwrap();
        std::fs::write(work.join(".malvin/gates"), "pytest tests\n").unwrap();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(work).unwrap();
        let mut state = super::PhaseState::fresh();
        super::observe_event(&mut state, &sdk_tool("shell", "error", "Run pytest tests"));
        assert_eq!(state.resolve().label(), "Fixing");
        super::observe_event(&mut state, &sdk_tool("shell", "complete", "Run echo ok"));
        assert_eq!(state.resolve().label(), "Fixing");
        state.checking_depth = 1;
        assert_eq!(state.resolve().label(), "Checking");
        super::observe_event(
            &mut state,
            &sdk_tool("shell", "complete", "Run pytest tests"),
        );
        state.checking_depth = 0;
        assert_eq!(state.resolve().label(), "Thinking");
        std::env::set_current_dir(old).unwrap();
    });
}
