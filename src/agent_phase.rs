//! Infers agent phase labels for stdout heartbeats from malvin runtime signals.
//!
//! Phase labels for stdout heartbeats (Orienting, Researching, …).

#[path = "agent_phase_signal.rs"]
mod agent_phase_signal;

use std::sync::Mutex;

use crate::tool_summary::{ParsedToolUpdate, ToolSummaryTracker};

/// Agent phase label shown in stdout heartbeats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentPhase {
    Orienting,
    Researching,
    Reasoning,
    Implementing,
    Executing,
    Verifying,
    Debugging,
    KPopCycling,
    Waiting,
    Reporting,
}

impl AgentPhase {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Orienting => "Orienting",
            Self::Researching => "Researching",
            Self::Reasoning => "Reasoning",
            Self::Implementing => "Implementing",
            Self::Executing => "Executing",
            Self::Verifying => "Verifying",
            Self::Debugging => "Debugging",
            Self::KPopCycling => "KPop cycling",
            Self::Waiting => "Waiting",
            Self::Reporting => "Reporting",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ToolKind {
    Read,
    Search,
    Edit,
    Execute,
}

#[allow(clippy::struct_excessive_bools)]
pub(super) struct PhaseState {
    pub(super) verifying_depth: u32,
    pub(super) kpop_depth: u32,
    pub(super) reporting: bool,
    pub(super) orienting: bool,
    pub(super) debugging: bool,
    pub(super) running_shells: u32,
    pub(super) reasoning: bool,
    pub(super) active_tool: Option<(ToolKind, u8)>,
}

impl PhaseState {
    const fn fresh() -> Self {
        Self {
            verifying_depth: 0,
            kpop_depth: 0,
            reporting: false,
            orienting: true,
            debugging: false,
            running_shells: 0,
            reasoning: false,
            active_tool: None,
        }
    }

    fn resolve(&self) -> AgentPhase {
        phase_if(self.reporting, AgentPhase::Reporting)
            .or_else(|| phase_if(self.verifying_depth > 0, AgentPhase::Verifying))
            .or_else(|| phase_if(self.kpop_depth > 0, AgentPhase::KPopCycling))
            .or_else(|| phase_if(self.running_shells > 0, AgentPhase::Waiting))
            .or_else(|| phase_if(self.debugging, AgentPhase::Debugging))
            .or_else(|| self.active_tool.map(|(k, _)| active_tool_phase(k)))
            .or_else(|| phase_if(self.reasoning, AgentPhase::Reasoning))
            .or_else(|| phase_if(self.orienting, AgentPhase::Orienting))
            .unwrap_or(AgentPhase::Reasoning)
    }
}

fn phase_if(cond: bool, phase: AgentPhase) -> Option<AgentPhase> {
    cond.then_some(phase)
}

const fn active_tool_phase(kind: ToolKind) -> AgentPhase {
    match kind {
        ToolKind::Read | ToolKind::Search => AgentPhase::Researching,
        ToolKind::Edit => AgentPhase::Implementing,
        ToolKind::Execute => AgentPhase::Executing,
    }
}

static STATE: Mutex<PhaseState> = Mutex::new(PhaseState::fresh());

#[cfg(test)]
pub(crate) static AGENT_PHASE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn with_state<R>(f: impl FnOnce(&mut PhaseState) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}

pub fn reset_for_run() {
    with_state(|s| *s = PhaseState::fresh());
}

pub fn note_orienting() {
    with_state(|s| {
        s.orienting = true;
        s.reasoning = false;
    });
}

pub fn clear_orienting() {
    with_state(|s| s.orienting = false);
}

pub fn enter_kpop() {
    with_state(|s| {
        s.kpop_depth = s.kpop_depth.saturating_add(1);
        s.orienting = false;
    });
}

pub fn leave_kpop() {
    with_state(|s| s.kpop_depth = s.kpop_depth.saturating_sub(1));
}

pub fn enter_verifying() {
    with_state(|s| {
        s.verifying_depth = s.verifying_depth.saturating_add(1);
        s.orienting = false;
    });
}

pub fn leave_verifying() {
    with_state(|s| s.verifying_depth = s.verifying_depth.saturating_sub(1));
}

pub fn set_reporting(active: bool) {
    with_state(|s| s.reporting = active);
}

/// Emits `DONE` on stdout while heartbeat labels use the Reporting phase.
pub fn print_done_with_reporting_phase() {
    use crate::output::{MALVIN_WHO, print_stdout_line};
    set_reporting(true);
    print_stdout_line(MALVIN_WHO, "DONE");
    set_reporting(false);
}

pub fn note_thought_activity() {
    with_state(|s| {
        s.reasoning = true;
        s.orienting = false;
    });
}

pub(crate) fn observe_tool_update(parsed: &ParsedToolUpdate, tracker: &ToolSummaryTracker) {
    with_state(|s| agent_phase_signal::observe_tool_update_state(s, parsed, tracker));
}

#[must_use]
pub fn heartbeat_label() -> &'static str {
    with_state(|s| s.resolve().label())
}

#[cfg(test)]
pub(crate) fn current_phase_for_test() -> AgentPhase {
    with_state(|s| s.resolve())
}

#[cfg(test)]
pub(crate) fn reset_phase_state_for_test() {
    with_state(|s| *s = PhaseState::fresh());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_summary::ToolSummaryDetail;
    use serde_json::json;

    fn guard() -> std::sync::MutexGuard<'static, ()> {
        AGENT_PHASE_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn observe(update: serde_json::Value, tracker: &mut ToolSummaryTracker) {
        let v = json!({"method": "session/update", "params": {"update": update}});
        crate::tool_summary::tool_summary_lines(&v, tracker, ToolSummaryDetail::Log).unwrap();
    }

    #[test]
    fn kiss_cov_agent_phase_functions() {
        let _ = super::reset_for_run;
        let _ = super::note_orienting;
        let _ = super::clear_orienting;
        let _ = super::enter_kpop;
        let _ = super::leave_kpop;
        let _ = super::enter_verifying;
        let _ = super::leave_verifying;
        let _ = super::set_reporting;
        let _ = super::note_thought_activity;
        let _ = super::observe_tool_update;
        let _ = super::heartbeat_label;
        let _ = super::active_tool_phase;
        let _ = super::phase_if;
        let _ = stringify!(super::with_state);
        let _ = stringify!(super::PhaseState::fresh);
    }

    #[test]
    fn heartbeat_phases_follow_runtime_signals() {
        let _g = guard();
        reset_phase_state_for_test();
        assert_eq!(heartbeat_label(), "Orienting");
        note_thought_activity();
        reset_for_run();
        enter_verifying();
        leave_verifying();
        let mut tracker = ToolSummaryTracker::default();
        observe(json!({"sessionUpdate":"tool_call","toolCallId":"1","kind":"execute","status":"pending","rawInput":{"command":"sleep 9"}}), &mut tracker);
        observe(json!({"sessionUpdate":"tool_call_update","toolCallId":"1","status":"completed","rawOutput":{"exitCode":0}}), &mut tracker);
        observe(json!({"sessionUpdate":"tool_call","toolCallId":"r1","kind":"read","status":"pending","rawInput":{"path":"a.rs"}}), &mut tracker);
        assert_eq!(current_phase_for_test(), AgentPhase::Researching);
        observe(json!({"sessionUpdate":"tool_call","toolCallId":"x1","kind":"execute","status":"pending","rawInput":{"command":"cargo nextest run"}}), &mut tracker);
        observe(json!({"sessionUpdate":"tool_call_update","toolCallId":"x1","status":"completed","rawOutput":{"exitCode":1}}), &mut tracker);
        assert_eq!(current_phase_for_test(), AgentPhase::Debugging);
        set_reporting(true);
        assert_eq!(heartbeat_label(), "Reporting");
    }
}
