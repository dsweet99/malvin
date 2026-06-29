//! MPC planning brief protocol (see `concepts_2.md` §5).
//!
//! When `mpc` is enabled, an MPC planner agent runs at the start of each outer gate-loop
//! iteration, appends planning sections to the user brief, logs hypotheses to
//! `_kpop/mpc_planner_log.md`, and the loop exits on `## MPC_DONE` plus passing gates.
//! Production references [`MpcPlanningBriefAspect`] at enforcement sites in `kpop_engine`.

/// One aspect of the MPC planning brief protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MpcPlanningBriefAspect {
    /// `mpc_enabled` reads workspace config.
    ConfigEnabled,
    /// Outer-loop iteration invokes the planner before the implementer.
    PlannerSessionHook,
    /// Agent appends Current State / Q&A / Phases to the user brief (prompt-driven).
    BriefAppendProtocol,
    /// `_kpop/mpc_planner_log.md` via `mpc_planner_exp_log_path`.
    HypothesisLogPath,
    /// `mpc_declared_done` / `user_brief_declares_mpc_done` parse `## MPC_DONE`.
    DoneMarkerDetection,
    /// `mpc_done_early_exit` and KPop-solved suppression when MPC is on.
    ExitGateIntegration,
}

impl MpcPlanningBriefAspect {
    /// All protocol aspects in stable concept order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::ConfigEnabled,
            Self::PlannerSessionHook,
            Self::BriefAppendProtocol,
            Self::HypothesisLogPath,
            Self::DoneMarkerDetection,
            Self::ExitGateIntegration,
        ]
    }

    /// Primary module/file that owns this aspect at runtime.
    #[must_use]
    pub const fn primary_module(self) -> &'static str {
        match self {
            Self::ConfigEnabled => "src/malvin_config_file/mod.rs",
            Self::PlannerSessionHook => "src/kpop_engine/run_loop.rs",
            Self::BriefAppendProtocol => "default_prompts/mpc_planner.md",
            Self::HypothesisLogPath => "src/kpop_engine/mpc_planner.rs",
            Self::DoneMarkerDetection => "src/kpop_progression/counters.rs",
            Self::ExitGateIntegration => "src/kpop_engine/run_loop_exit.rs",
        }
    }
}

#[cfg(test)]
#[path = "mpc_planning_brief_tests.rs"]
mod mpc_planning_brief_tests;
