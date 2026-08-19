use crate::agent_phase::{PhaseState, ToolKind};
use crate::tool_summary::{
    ParsedToolUpdate, TOOL_PHASE_DONE, TOOL_PHASE_RUNNING, TOOL_PHASE_START, ToolSummaryTracker,
    execute_effective_exit, execute_stdout_failed,
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

fn observe_execute(
    state: &mut PhaseState,
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
) {
    match parsed.phase {
        TOOL_PHASE_START => {
            state.running_shells = state.running_shells.saturating_add(1);
            state.active_tool = Some((ToolKind::Execute, parsed.phase));
            crate::herdr::notify_working();
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
        .or_else(|| {
            tracker
                .record(&parsed.id)
                .and_then(|r| r.command.as_deref())
        })
        .or_else(|| {
            parsed
                .title
                .strip_prefix('`')
                .and_then(|t| t.strip_suffix('`'))
        })
        .unwrap_or("");
    std::env::current_dir()
        .is_ok_and(|wd| crate::repo_gates::command_matches_malvin_checks_gate(cmd, &wd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_phase::PhaseState;
    use crate::tool_summary::{ToolSummaryTracker, parse_tool_update};
    use serde_json::json;

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
        assert_eq!(state.active_tool.map(|(k, _)| k), Some(ToolKind::Read));
    }

    #[test]
    fn observe_execute_failed_gate_command_sets_debugging() {
        crate::test_utils::with_isolated_home(|w| {
            std::fs::create_dir_all(w.join(".malvin")).expect("mkdir");
            std::fs::write(w.join(".malvin/checks"), "pytest tests\n").expect("checks");
            let old = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(w).expect("chdir");

            let mut state = PhaseState::fresh();
            let tracker = ToolSummaryTracker::default();
            let start = json!({
                "method": "session/update",
                "params": {"update": {
                    "sessionUpdate": "tool_call",
                    "toolCallId": "ex1",
                    "kind": "execute",
                    "status": "pending",
                    "rawInput": {"command": "pytest tests"}
                }}
            });
            let running = json!({
                "method": "session/update",
                "params": {"update": {
                    "sessionUpdate": "tool_call_update",
                    "toolCallId": "ex1",
                    "kind": "execute",
                    "status": "in_progress"
                }}
            });
            let done = json!({
                "method": "session/update",
                "params": {"update": {
                    "sessionUpdate": "tool_call_update",
                    "toolCallId": "ex1",
                    "kind": "execute",
                    "status": "completed",
                    "rawOutput": {"exitCode": 1, "stdout": "FAILED\n", "stderr": ""}
                }}
            });
            observe_tool_update_state(
                &mut state,
                &parse_tool_update(&start).expect("start"),
                &tracker,
            );
            assert_eq!(state.running_shells, 1);
            assert_eq!(
                state.active_tool,
                Some((ToolKind::Execute, TOOL_PHASE_START))
            );
            observe_tool_update_state(
                &mut state,
                &parse_tool_update(&running).expect("running"),
                &tracker,
            );
            assert_eq!(
                state.active_tool,
                Some((ToolKind::Execute, TOOL_PHASE_RUNNING))
            );
            observe_tool_update_state(
                &mut state,
                &parse_tool_update(&done).expect("done"),
                &tracker,
            );
            assert_eq!(state.running_shells, 0);
            assert!(state.active_tool.is_none());
            assert!(
                state.debugging,
                "failed checks-file command should enter debugging"
            );
            std::env::set_current_dir(old).expect("restore cwd");
        });
    }

    #[test]
    fn observe_non_execute_done_clears_matching_active_tool() {
        let mut state = PhaseState::fresh();
        let tracker = ToolSummaryTracker::default();
        let start = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "e1",
                "kind": "edit",
                "status": "pending",
                "rawInput": {"path": "a.rs"}
            }}
        });
        let done = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "e1",
                "kind": "edit",
                "status": "completed"
            }}
        });
        observe_tool_update_state(
            &mut state,
            &parse_tool_update(&start).expect("start"),
            &tracker,
        );
        assert_eq!(state.active_tool.map(|(k, _)| k), Some(ToolKind::Edit));
        observe_tool_update_state(
            &mut state,
            &parse_tool_update(&done).expect("done"),
            &tracker,
        );
        assert!(state.active_tool.is_none());
    }
}
