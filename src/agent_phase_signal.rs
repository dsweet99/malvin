//! Tool-call signal hooks for [`crate::agent_phase`] heartbeat labels.

use crate::agent_phase::{PhaseState, ToolKind};
use crate::tool_summary::{
    execute_effective_exit, execute_stdout_failed, ParsedToolUpdate, ToolSummaryTracker,
    TOOL_PHASE_DONE, TOOL_PHASE_RUNNING, TOOL_PHASE_START,
};

pub(super) fn observe_tool_update_state(
    state: &mut PhaseState,
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
) {
    let Some(kind) = tool_kind_for(parsed, tracker) else {
        return;
    };
    state.orienting = false;
    state.reasoning = false;
    match kind {
        ToolKind::Execute => observe_execute(state, parsed, tracker),
        _ => observe_non_execute(state, kind, parsed.phase),
    }
}

fn tool_kind_for(parsed: &ParsedToolUpdate, tracker: &ToolSummaryTracker) -> Option<ToolKind> {
    let kind = tracker
        .record(&parsed.id)
        .map_or(parsed.kind.as_str(), |r| r.kind.as_str());
    match kind {
        "read" => Some(ToolKind::Read),
        "search" => Some(ToolKind::Search),
        "edit" => Some(ToolKind::Edit),
        "execute" => Some(ToolKind::Execute),
        _ => None,
    }
}

fn observe_execute(state: &mut PhaseState, parsed: &ParsedToolUpdate, tracker: &ToolSummaryTracker) {
    match parsed.phase {
        TOOL_PHASE_START => {
            state.running_shells = state.running_shells.saturating_add(1);
            state.active_tool = Some((ToolKind::Execute, parsed.phase));
        }
        TOOL_PHASE_RUNNING => state.active_tool = Some((ToolKind::Execute, parsed.phase)),
        TOOL_PHASE_DONE => {
            state.running_shells = state.running_shells.saturating_sub(1);
            if execute_failed(parsed) && execute_looks_like_test(parsed, tracker) {
                state.debugging = true;
            }
            state.active_tool = None;
        }
        _ => {}
    }
}

fn observe_non_execute(state: &mut PhaseState, kind: ToolKind, phase: u8) {
    if phase == TOOL_PHASE_DONE {
        if state.active_tool.is_some_and(|(k, _)| k == kind) {
            state.active_tool = None;
        }
        return;
    }
    state.active_tool = Some((kind, phase));
}

fn execute_failed(parsed: &ParsedToolUpdate) -> bool {
    let raw = parsed.raw_output.as_ref();
    let exit = execute_effective_exit(parsed, raw);
    execute_stdout_failed(parsed, exit, raw)
}

fn execute_looks_like_test(parsed: &ParsedToolUpdate, tracker: &ToolSummaryTracker) -> bool {
    let cmd = parsed
        .command
        .as_deref()
        .or_else(|| tracker.record(&parsed.id).and_then(|r| r.command.as_deref()))
        .or_else(|| parsed.title.strip_prefix('`').and_then(|t| t.strip_suffix('`')))
        .unwrap_or("");
    std::env::current_dir().is_ok_and(|wd| {
        crate::repo_gates::command_matches_malvin_checks_gate(cmd, &wd)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_phase::PhaseState;
    use crate::tool_summary::{parse_tool_update, ToolSummaryTracker};
    use serde_json::json;

    #[test]
    fn kiss_cov_signal_privates() {
    }

    #[test]
    fn observe_tool_update_state_handles_read_start() {
        let mut state = PhaseState::fresh();
        let v = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "s1",
                "kind": "read",
                "status": "pending",
                "rawInput": {"path": "x.rs"}
            }}
        });
        let parsed = parse_tool_update(&v).expect("parsed");
        let tracker = ToolSummaryTracker::default();
        observe_tool_update_state(&mut state, &parsed, &tracker);
        assert!(!state.orienting);
    }
}
