use super::agent_phase_signal::PhaseSideEffect;
use super::{PhaseState, apply_phase_side_effect, with_state};
use crate::backends::cursor_sdk::protocol::BridgeEvent;

pub(crate) fn observe_stream_event(event: &BridgeEvent) {
    let effect = with_state(|state| observe_event(state, event));
    apply_phase_side_effect(effect);
}

fn observe_event(state: &mut PhaseState, event: &BridgeEvent) -> PhaseSideEffect {
    match event {
        BridgeEvent::Assistant { text } | BridgeEvent::Thinking { text } if !text.is_empty() => {
            state.starting = false;
        }
        BridgeEvent::ToolCall {
            phase,
            name,
            summary,
            tool_call_id,
            ..
        } => {
            state.starting = false;
            let id = tool_call_id
                .as_deref()
                .filter(|id| !id.is_empty())
                .unwrap_or("");
            return observe_tool(state, id, phase, (name.as_deref(), summary.as_deref()));
        }
        BridgeEvent::RunDone { .. } | BridgeEvent::Fatal { .. } => {
            state.starting = false;
            state.running_shells.clear();
            state.open_reads.clear();
            state.open_edits.clear();
            state.active_tool = None;
        }
        _ => {}
    }
    PhaseSideEffect::None
}

fn observe_tool(
    state: &mut PhaseState,
    id: &str,
    phase: &str,
    (name, summary): (Option<&str>, Option<&str>),
) -> PhaseSideEffect {
    if matches!(phase, "complete" | "end" | "done" | "error") {
        state.running_shells.remove(id);
        state.open_reads.remove(id);
        state.open_edits.remove(id);
        observe_check_result(state, summary, phase);
    } else if matches!(phase, "start" | "running") {
        match crate::tool_summary::sdk_tool_kind(name) {
            "read" | "search" => {
                state.open_reads.insert(id.to_string());
            }
            "edit" => {
                state.open_edits.insert(id.to_string());
            }
            "execute" => {
                state.running_shells.insert(id.to_string());
            }
            _ => {}
        }
        return PhaseSideEffect::NotifyWorking;
    }
    PhaseSideEffect::None
}

fn observe_check_result(state: &mut PhaseState, summary: Option<&str>, phase: &str) {
    let Some(command) = summary.and_then(|summary| summary.strip_prefix("Run ")) else {
        return;
    };
    if std::env::current_dir()
        .is_ok_and(|wd| crate::repo_gates::command_matches_malvin_checks_gate(command, &wd))
    {
        state.fixing = phase == "error";
    }
}

#[cfg(test)]
#[path = "agent_phase_sdk_tests.rs"]
mod tests;
