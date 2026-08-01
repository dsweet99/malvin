//! Mini loop terminal reasons and records (`mini.md` audit contract).

pub use crate::coder_prompt_phase::MiniPhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniTerminalReason {
    FencelessComplete,
    MiniDoneOutsideFence,
    FencelessPremature,
    BudgetExhaustedBeforeClassification,
    BudgetExhaustedAfterBashOnLastHttpTurn,
    BudgetExhaustedBashExecs,
    HttpRetryExhausted,
    GateIterationExhausted,
    ContextOverflow,
}

impl MiniTerminalReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FencelessComplete => "fenceless_complete",
            Self::MiniDoneOutsideFence => "mini_done_outside_fence",
            Self::FencelessPremature => "fenceless_premature",
            Self::BudgetExhaustedBeforeClassification => "budget_exhausted_before_classification",
            Self::BudgetExhaustedAfterBashOnLastHttpTurn => "budget_exhausted_after_bash_on_last_http_turn",
            Self::BudgetExhaustedBashExecs => "budget_exhausted_bash_execs",
            Self::HttpRetryExhausted => "http_retry_exhausted",
            Self::GateIterationExhausted => "gate_iteration_exhausted",
            Self::ContextOverflow => "context_overflow",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiniTerminalRecord {
    pub reason: MiniTerminalReason,
    pub http_turn_count: u32,
    pub bash_exec_count: u32,
    pub phase_at_exit: MiniPhase,
}

impl MiniTerminalRecord {
    #[must_use]
    pub const fn new(
        reason: MiniTerminalReason,
        http_turn_count: u32,
        bash_exec_count: u32,
        phase_at_exit: MiniPhase,
    ) -> Self {
        Self {
            reason,
            http_turn_count,
            bash_exec_count,
            phase_at_exit,
        }
    }
}
