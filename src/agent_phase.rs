//! Infers agent phase labels for stdout heartbeats from malvin runtime signals.
//!
//! Phase labels for stdout heartbeats (Orienting, Researching, …).

#[path = "agent_phase_signal.rs"]
mod agent_phase_signal;

use std::sync::Mutex;

use crate::tool_summary::{ParsedToolUpdate, ToolSummaryTracker, TOOL_PHASE_RUNNING};

/// Agent phase label shown in stdout heartbeats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AgentPhase {
    Orienting = 0,
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

const PHASE_LABELS: [&str; 10] = [
    "Orienting",
    "Researching",
    "Reasoning",
    "Implementing",
    "Executing",
    "Verifying",
    "Debugging",
    "KPop cycling",
    "Waiting",
    "Reporting",
];

impl AgentPhase {
    #[must_use]
    pub const fn label(self) -> &'static str {
        PHASE_LABELS[self as usize]
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

/// Mini HTTP completion is in-flight (`OpenRouter` await).
pub fn note_mini_llm_request() {
    note_thought_activity();
}

/// Mini bash fence is executing synchronously.
pub fn note_mini_bash_exec() {
    with_state(|s| {
        s.orienting = false;
        s.reasoning = false;
        s.active_tool = Some((ToolKind::Execute, TOOL_PHASE_RUNNING));
    });
}

/// Mini bash fence finished; mirrors execute tool-call completion signals.
pub fn note_mini_bash_exec_done(exit_code: i32, command: &str) {
    with_state(|s| {
        if exit_code != 0
            && std::env::current_dir().is_ok_and(|wd| {
                crate::repo_gates::command_matches_malvin_checks_gate(command, &wd)
            })
        {
            s.debugging = true;
        }
        s.active_tool = None;
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
pub(crate) mod kiss_cov {
    pub(crate) use super::ToolKind;
    use super::{active_tool_phase, phase_if};

    #[must_use]
    pub(crate) fn witness_tool_kinds() -> [ToolKind; 4] {
        [
            ToolKind::Read,
            ToolKind::Search,
            ToolKind::Edit,
            ToolKind::Execute,
        ]
    }

    #[must_use]
    pub(crate) fn witness_phase_if(cond: bool) -> Option<super::AgentPhase> {
        phase_if(cond, super::AgentPhase::Waiting)
    }

    #[must_use]
    pub(crate) fn witness_active_tool_phase(kind: ToolKind) -> super::AgentPhase {
        active_tool_phase(kind)
    }
}

