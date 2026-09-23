use super::map_event::map_npm_pi_event;
use super::session_turn::{TurnState, feed_mapped_bridge_events};
use crate::acp::AgentIoOptions;
use crate::bridge_sdk::{DrainIdleLabels, DrainIdleTurn, StreamLog, tools_in_flight};
use crate::sdk_drain_timeout::{sdk_drain_idle_max_turn, sdk_drain_idle_max_wait};
use std::time::Duration;

fn quiet_log() -> StreamLog {
    StreamLog::new(AgentIoOptions {
        no_tee: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    })
}

#[test]
fn issue_43_tool_execution_start_extends_npm_pi_turn_cap() {
    let log = quiet_log();
    let mut turn = DrainIdleTurn::new();
    let before = turn.clock.max_deadline();
    let mut state = TurnState::default();
    let start = serde_json::json!({
        "type": "tool_execution_start",
        "toolCallId": "t1",
        "toolName": "bash",
        "args": {"command": "sleep 1"}
    });
    let events = map_npm_pi_event(&start, &mut state);
    feed_mapped_bridge_events(&log, &mut turn, &events);
    assert!(
        turn.clock.max_deadline() > before,
        "issue #43: mapped tool_execution_start must extend the active turn deadline"
    );
    assert!(
        tools_in_flight(&log),
        "issue #43: open tool must set tools_in_flight for StillBusy turn-cap extension"
    );
}

#[test]
fn issue_43_turn_budget_error_reports_active_deadline() {
    let idle = Duration::from_mins(10);
    let clock = crate::bridge_sdk::DrainIdleClock::new(idle);
    let active = sdk_drain_idle_max_wait(idle);
    let ceiling = sdk_drain_idle_max_turn(idle);
    assert_eq!(
        clock.turn_limit(),
        active,
        "issue #43: turn_limit must be the active deadline (2×idle), not the 10×idle ceiling"
    );
    assert_ne!(clock.turn_limit(), ceiling);
    let labels = DrainIdleLabels {
        prefix: "npm pi rpc timed out",
        waiting_for: "npm pi event",
    };
    let err = labels.turn_budget_error(Duration::from_mins(20), clock.turn_limit());
    assert!(err.message.contains("limit 1200s"), "{err:?}");
    assert!(!err.message.contains("limit 6000s"), "{err:?}");
}
