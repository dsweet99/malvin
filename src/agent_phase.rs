#[path = "agent_phase_signal.rs"]
mod agent_phase_signal;

use std::collections::HashSet;
use std::sync::Mutex;

use crate::tool_summary::{ParsedToolUpdate, ToolSummaryTracker};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AgentPhase {
    Starting = 0,
    Reading,
    Editing,
    Running,
    Checking,
    Fixing,
    Thinking,
}

const PHASE_LABELS: [&str; 7] = [
    "Starting", "Reading", "Editing", "Running", "Checking", "Fixing", "Thinking",
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

pub(super) struct PhaseState {
    pub(super) checking_depth: u32,
    pub(super) starting: bool,
    pub(super) fixing: bool,
    pub(super) running_shells: u32,
    pub(super) active_tool: Option<(ToolKind, u8)>,
    open_edits: HashSet<String>,
    open_reads: HashSet<String>,
}

impl PhaseState {
    fn fresh() -> Self {
        Self {
            checking_depth: 0,
            starting: true,
            fixing: false,
            running_shells: 0,
            active_tool: None,
            open_edits: HashSet::new(),
            open_reads: HashSet::new(),
        }
    }

    fn edit_in_flight(&self) -> bool {
        !self.open_edits.is_empty()
            || self
                .active_tool
                .is_some_and(|(k, _)| active_tool_phase(k) == AgentPhase::Editing)
    }

    fn read_in_flight(&self) -> bool {
        !self.open_reads.is_empty()
            || self
                .active_tool
                .is_some_and(|(k, _)| active_tool_phase(k) == AgentPhase::Reading)
    }

    fn shell_in_flight(&self) -> bool {
        self.running_shells > 0
            || self
                .active_tool
                .is_some_and(|(k, _)| active_tool_phase(k) == AgentPhase::Running)
    }

    fn resolve(&self) -> AgentPhase {
        phase_if(self.checking_depth > 0, AgentPhase::Checking)
            .or_else(|| phase_if(self.shell_in_flight(), AgentPhase::Running))
            .or_else(|| phase_if(self.edit_in_flight(), AgentPhase::Editing))
            .or_else(|| phase_if(self.read_in_flight(), AgentPhase::Reading))
            .or_else(|| phase_if(self.fixing, AgentPhase::Fixing))
            .or_else(|| phase_if(self.starting, AgentPhase::Starting))
            .unwrap_or(AgentPhase::Thinking)
    }
}

fn phase_if(cond: bool, phase: AgentPhase) -> Option<AgentPhase> {
    cond.then_some(phase)
}

const fn active_tool_phase(kind: ToolKind) -> AgentPhase {
    match kind {
        ToolKind::Read | ToolKind::Search => AgentPhase::Reading,
        ToolKind::Edit => AgentPhase::Editing,
        ToolKind::Execute => AgentPhase::Running,
    }
}

static STATE: std::sync::LazyLock<Mutex<PhaseState>> =
    std::sync::LazyLock::new(|| Mutex::new(PhaseState::fresh()));

#[cfg(test)]
pub(crate) static AGENT_PHASE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn with_state<R>(f: impl FnOnce(&mut PhaseState) -> R) -> R {
    let mut guard = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}

pub fn reset_for_run() {
    with_state(|s| *s = PhaseState::fresh());
}

pub fn note_starting() {
    with_state(|s| s.starting = true);
}

pub fn enter_checking() {
    with_state(|s| {
        s.checking_depth = s.checking_depth.saturating_add(1);
    });
    apply_phase_side_effect(agent_phase_signal::PhaseSideEffect::NotifyWorking);
}

pub fn leave_checking() {
    with_state(|s| s.checking_depth = s.checking_depth.saturating_sub(1));
}

pub(crate) fn observe_tool_update(parsed: &ParsedToolUpdate, tracker: &ToolSummaryTracker) {
    let effect = with_state(|s| agent_phase_signal::observe_tool_update_state(s, parsed, tracker));
    apply_phase_side_effect(effect);
}

fn apply_phase_side_effect(effect: agent_phase_signal::PhaseSideEffect) {
    if matches!(effect, agent_phase_signal::PhaseSideEffect::NotifyWorking) {
        crate::herdr::notify_working();
    }
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
        phase_if(cond, super::AgentPhase::Running)
    }

    #[must_use]
    pub(crate) fn witness_active_tool_phase(kind: ToolKind) -> super::AgentPhase {
        active_tool_phase(kind)
    }
}
