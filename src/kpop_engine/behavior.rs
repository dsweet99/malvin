#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KPopHardConstraintsExit {
    /// Two consecutive `## KPOP_SOLVED` markers and passing gates; restore checks each turn.
    CodeTidy,
    /// One `## KPOP_SOLVED` and valid checks file; do not restore `.malvin/checks` between turns.
    InitDiscovery,
}

#[derive(Clone, Copy)]
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
    pub const INIT: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        skip_workspace_quality_gates: false,
        exit: KPopHardConstraintsExit::InitDiscovery,
    };
    pub const DELIGHT: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        skip_workspace_quality_gates: true,
        exit: KPopHardConstraintsExit::CodeTidy,
    };
    pub const EXPLAIN: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        skip_workspace_quality_gates: true,
        exit: KPopHardConstraintsExit::CodeTidy,
    };
    pub const REVISE: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        skip_workspace_quality_gates: true,
        exit: KPopHardConstraintsExit::CodeTidy,
    };

    #[must_use]
    pub const fn consecutive_kpop_solved_to_exit(self) -> usize {
        match self.exit {
            KPopHardConstraintsExit::CodeTidy => 2,
            KPopHardConstraintsExit::InitDiscovery => 1,
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
    fn init_discovery_behavior_differs_from_code() {
        assert_eq!(KPopHardConstraints::INIT.exit, KPopHardConstraintsExit::InitDiscovery);
        assert_eq!(KPopHardConstraints::CODE.exit, KPopHardConstraintsExit::CodeTidy);
    }

    #[test]
    fn code_tidy_exit_policy_requires_two_solved_and_passing_gates() {
        assert_eq!(KPopHardConstraints::CODE.consecutive_kpop_solved_to_exit(), 2);
        assert_eq!(KPopHardConstraints::TIDY.consecutive_kpop_solved_to_exit(), 2);
        assert!(KPopHardConstraints::CODE.require_passing_gates_for_exit());
        assert!(KPopHardConstraints::TIDY.require_passing_gates_for_exit());
        const { assert!(!KPopHardConstraints::CODE.skip_workspace_quality_gates); }
        const { assert!(!KPopHardConstraints::TIDY.skip_workspace_quality_gates); }
    }

    #[test]
    fn init_discovery_exit_policy_requires_one_solved_without_gate_pass() {
        assert_eq!(KPopHardConstraints::INIT.consecutive_kpop_solved_to_exit(), 1);
        assert!(!KPopHardConstraints::INIT.require_passing_gates_for_exit());
    }

    #[test]
    fn explain_behavior_matches_delight_exit_policy() {
        assert_eq!(KPopHardConstraints::EXPLAIN.exit, KPopHardConstraintsExit::CodeTidy);
        assert_eq!(
            KPopHardConstraints::EXPLAIN.consecutive_kpop_solved_to_exit(),
            KPopHardConstraints::DELIGHT.consecutive_kpop_solved_to_exit(),
        );
        assert!(KPopHardConstraints::EXPLAIN.require_passing_gates_for_exit());
        const { assert!(KPopHardConstraints::EXPLAIN.skip_workspace_quality_gates); }
        const { assert!(KPopHardConstraints::DELIGHT.skip_workspace_quality_gates); }
    }

    #[test]
    fn revise_behavior_matches_explain_exit_policy() {
        assert_eq!(KPopHardConstraints::REVISE.exit, KPopHardConstraintsExit::CodeTidy);
        assert_eq!(
            KPopHardConstraints::REVISE.consecutive_kpop_solved_to_exit(),
            KPopHardConstraints::EXPLAIN.consecutive_kpop_solved_to_exit(),
        );
        assert!(KPopHardConstraints::REVISE.require_passing_gates_for_exit());
        const { assert!(KPopHardConstraints::REVISE.skip_workspace_quality_gates); }
    }

    #[test]
    fn delight_behavior_always_runs_kpop_and_exits_on_two_consecutive_solved() {
        assert_eq!(
            KPopHardConstraints::DELIGHT.skip_kpop_on_initial_pass,
            KPopHardConstraints::CODE.skip_kpop_on_initial_pass,
        );
        assert_ne!(
            KPopHardConstraints::DELIGHT.skip_kpop_on_initial_pass,
            KPopHardConstraints::TIDY.skip_kpop_on_initial_pass,
        );
        assert_eq!(KPopHardConstraints::DELIGHT.exit, KPopHardConstraintsExit::CodeTidy);
        assert_eq!(KPopHardConstraints::DELIGHT.consecutive_kpop_solved_to_exit(), 2);
        assert_ne!(
            KPopHardConstraints::DELIGHT.consecutive_kpop_solved_to_exit(),
            KPopHardConstraints::INIT.consecutive_kpop_solved_to_exit(),
        );
    }
}
