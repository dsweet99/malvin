#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KPopHardConstraintsExit {
    /// `CodeTidy` = passing gates; restore checks each turn.
    CodeTidy,
    /// `ChecksDiscovery` = valid checks file on disk; do not restore `.malvin/checks` between turns.
    ChecksDiscovery,
}

#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct KPopHardConstraints {
    pub skip_kpop_on_initial_pass: bool,
    pub recheck_gates_after_exhausted: bool,
    pub skip_workspace_quality_gates: bool,
    pub exit: KPopHardConstraintsExit,
}

impl KPopHardConstraints {
    pub const CODE: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: true,
        skip_workspace_quality_gates: false,
        exit: KPopHardConstraintsExit::CodeTidy,
    };
    pub const TIDY: Self = Self {
        skip_kpop_on_initial_pass: true,
        recheck_gates_after_exhausted: false,
        skip_workspace_quality_gates: false,
        exit: KPopHardConstraintsExit::CodeTidy,
    };
    pub const CHECKS_DISCOVERY: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        skip_workspace_quality_gates: false,
        exit: KPopHardConstraintsExit::ChecksDiscovery,
    };
    /// Historical DELIGHT gate-engine profile (delight now uses the default router).
    #[allow(dead_code)]
    pub const DELIGHT: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        skip_workspace_quality_gates: true,
        exit: KPopHardConstraintsExit::CodeTidy,
    };
    /// Historical EXPLAIN gate-engine profile (explain now uses Review→Plan→Work).
    #[allow(dead_code)]
    pub const EXPLAIN: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        skip_workspace_quality_gates: true,
        exit: KPopHardConstraintsExit::CodeTidy,
    };

    #[must_use]
    pub const fn with_workspace_quality_gates(mut self, gates: bool) -> Self {
        if !gates {
            self.skip_workspace_quality_gates = true;
        }
        self
    }

    #[must_use]
    pub const fn consecutive_kpop_solved_to_exit(self) -> usize {
        match self.exit {
            KPopHardConstraintsExit::CodeTidy => 2,
            KPopHardConstraintsExit::ChecksDiscovery => 1,
        }
    }

    #[must_use]
    pub const fn require_passing_gates_for_exit(self) -> bool {
        matches!(self.exit, KPopHardConstraintsExit::CodeTidy)
    }

    #[must_use]
    pub const fn restore_malvin_checks_after_session(self) -> bool {
        matches!(self.exit, KPopHardConstraintsExit::CodeTidy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_loop_behavior_code_and_tidy_differ() {
        assert_ne!(
            (
                KPopHardConstraints::CODE.skip_kpop_on_initial_pass,
                KPopHardConstraints::CODE.recheck_gates_after_exhausted,
            ),
            (
                KPopHardConstraints::TIDY.skip_kpop_on_initial_pass,
                KPopHardConstraints::TIDY.recheck_gates_after_exhausted,
            ),
        );
    }

    #[test]
    fn checks_discovery_behavior_differs_from_code() {
        assert_eq!(
            KPopHardConstraints::CHECKS_DISCOVERY.exit,
            KPopHardConstraintsExit::ChecksDiscovery
        );
        assert_eq!(KPopHardConstraints::CODE.exit, KPopHardConstraintsExit::CodeTidy);
    }

    #[test]
    fn code_tidy_exit_policy_requires_passing_gates() {
        assert!(KPopHardConstraints::CODE.require_passing_gates_for_exit());
        assert!(KPopHardConstraints::TIDY.require_passing_gates_for_exit());
        const { assert!(!KPopHardConstraints::CODE.skip_workspace_quality_gates); }
        const { assert!(!KPopHardConstraints::TIDY.skip_workspace_quality_gates); }
    }

    #[test]
    fn cli_gates_only_disable_behavior() {
        assert!(
            KPopHardConstraints::CODE
                .with_workspace_quality_gates(false)
                .skip_workspace_quality_gates
        );
        assert!(
            !KPopHardConstraints::CODE
                .with_workspace_quality_gates(true)
                .skip_workspace_quality_gates
        );
        assert!(
            KPopHardConstraints::DELIGHT
                .with_workspace_quality_gates(true)
                .skip_workspace_quality_gates
        );
    }

    #[test]
    fn checks_discovery_exit_policy_allows_done_without_gate_pass() {
        assert!(!KPopHardConstraints::CHECKS_DISCOVERY.require_passing_gates_for_exit());
    }

    #[test]
    fn explain_behavior_matches_delight_exit_policy() {
        assert_eq!(KPopHardConstraints::EXPLAIN.exit, KPopHardConstraintsExit::CodeTidy);
        assert!(KPopHardConstraints::EXPLAIN.require_passing_gates_for_exit());
        const { assert!(KPopHardConstraints::EXPLAIN.skip_workspace_quality_gates); }
        const { assert!(KPopHardConstraints::DELIGHT.skip_workspace_quality_gates); }
    }

    #[test]
    fn delight_behavior_always_runs_kpop() {
        assert_eq!(
            KPopHardConstraints::DELIGHT.skip_kpop_on_initial_pass,
            KPopHardConstraints::CODE.skip_kpop_on_initial_pass,
        );
        assert_ne!(
            KPopHardConstraints::DELIGHT.skip_kpop_on_initial_pass,
            KPopHardConstraints::TIDY.skip_kpop_on_initial_pass,
        );
        assert_eq!(KPopHardConstraints::DELIGHT.exit, KPopHardConstraintsExit::CodeTidy);
        assert!(KPopHardConstraints::DELIGHT.require_passing_gates_for_exit());
    }
}
