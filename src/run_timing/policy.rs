use std::sync::atomic::{AtomicBool, Ordering};

pub const RUN_TIMING_JSON_FILE: &str = "run_timing.json";

pub const RUN_TIMING_SUMMARY_PREFIX: &str = "TIMING: ";

static SUPPRESS_STDOUT_FOOTNOTES: AtomicBool = AtomicBool::new(false);

pub fn set_suppress_stdout_footnotes(suppress: bool) {
    SUPPRESS_STDOUT_FOOTNOTES.store(suppress, Ordering::Relaxed);
}

pub(crate) fn stdout_footnotes_suppressed() -> bool {
    SUPPRESS_STDOUT_FOOTNOTES.load(Ordering::Relaxed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingPhase {
    Implement,
}

pub const TOOL_CALL_TYPE_MS_KEYS: [&str; 5] = ["read", "search", "edit", "execute", "other"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum AcpStepProxy {
    #[default]
    Idle,
    OpenBatch,
    TrailingAssistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CostPolicy {
    #[default]
    EstimateFromRates,
    Zero,
}

#[must_use]
pub const fn cost_policy_for_model(_model: &str) -> CostPolicy {
    CostPolicy::EstimateFromRates
}
