#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GateKpopExitPolicy {
    /// Two consecutive `## KPOP_SOLVED` markers and passing gates; restore checks each turn.
    CodeTidy,
    /// One `## KPOP_SOLVED` and valid checks file; do not restore `.malvin/checks` between turns.
    InitDiscovery,
}

#[derive(Clone, Copy)]
pub(crate) struct GateLoopBehavior {
    pub skip_kpop_on_initial_pass: bool,
    pub recheck_gates_after_exhausted: bool,
    pub exit: GateKpopExitPolicy,
}

impl GateLoopBehavior {
    pub const CODE: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: true,
        exit: GateKpopExitPolicy::CodeTidy,
    };
    pub const TIDY: Self = Self {
        skip_kpop_on_initial_pass: true,
        recheck_gates_after_exhausted: false,
        exit: GateKpopExitPolicy::CodeTidy,
    };
    pub const INIT: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        exit: GateKpopExitPolicy::InitDiscovery,
    };
    pub const DELIGHT: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        exit: GateKpopExitPolicy::InitDiscovery,
    };
    pub const EXPLAIN: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        exit: GateKpopExitPolicy::InitDiscovery,
    };
    pub const REVISE: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: false,
        exit: GateKpopExitPolicy::InitDiscovery,
    };

    #[must_use]
    pub const fn consecutive_kpop_solved_to_exit(self) -> usize {
        match self.exit {
            GateKpopExitPolicy::CodeTidy => 2,
            GateKpopExitPolicy::InitDiscovery => 1,
        }
    }

    #[must_use]
    pub const fn require_passing_gates_for_exit(self) -> bool {
        matches!(self.exit, GateKpopExitPolicy::CodeTidy)
    }

    #[must_use]
    pub const fn restore_malvin_checks_after_session(self) -> bool {
        matches!(self.exit, GateKpopExitPolicy::CodeTidy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_loop_behavior_code_and_tidy_differ() {
        assert_ne!(
            (
                GateLoopBehavior::CODE.skip_kpop_on_initial_pass,
                GateLoopBehavior::CODE.recheck_gates_after_exhausted,
            ),
            (
                GateLoopBehavior::TIDY.skip_kpop_on_initial_pass,
                GateLoopBehavior::TIDY.recheck_gates_after_exhausted,
            ),
        );
    }

    #[test]
    fn init_discovery_behavior_differs_from_code() {
        assert_eq!(GateLoopBehavior::INIT.exit, GateKpopExitPolicy::InitDiscovery);
        assert_eq!(GateLoopBehavior::CODE.exit, GateKpopExitPolicy::CodeTidy);
    }

    #[test]
    fn code_tidy_exit_policy_requires_two_solved_and_passing_gates() {
        assert_eq!(GateLoopBehavior::CODE.consecutive_kpop_solved_to_exit(), 2);
        assert_eq!(GateLoopBehavior::TIDY.consecutive_kpop_solved_to_exit(), 2);
        assert!(GateLoopBehavior::CODE.require_passing_gates_for_exit());
        assert!(GateLoopBehavior::TIDY.require_passing_gates_for_exit());
    }

    #[test]
    fn init_discovery_exit_policy_requires_one_solved_without_gate_pass() {
        assert_eq!(GateLoopBehavior::INIT.consecutive_kpop_solved_to_exit(), 1);
        assert!(!GateLoopBehavior::INIT.require_passing_gates_for_exit());
    }

    #[test]
    fn explain_behavior_matches_delight_exit_policy() {
        assert_eq!(GateLoopBehavior::EXPLAIN.exit, GateKpopExitPolicy::InitDiscovery);
        assert_eq!(
            GateLoopBehavior::EXPLAIN.consecutive_kpop_solved_to_exit(),
            GateLoopBehavior::DELIGHT.consecutive_kpop_solved_to_exit(),
        );
        assert!(!GateLoopBehavior::EXPLAIN.require_passing_gates_for_exit());
    }

    #[test]
    fn revise_behavior_matches_explain_exit_policy() {
        assert_eq!(GateLoopBehavior::REVISE.exit, GateKpopExitPolicy::InitDiscovery);
        assert_eq!(
            GateLoopBehavior::REVISE.consecutive_kpop_solved_to_exit(),
            GateLoopBehavior::EXPLAIN.consecutive_kpop_solved_to_exit(),
        );
        assert!(!GateLoopBehavior::REVISE.require_passing_gates_for_exit());
    }

    #[test]
    fn delight_behavior_always_runs_kpop_and_exits_on_one_solved() {
        assert_eq!(
            GateLoopBehavior::DELIGHT.skip_kpop_on_initial_pass,
            GateLoopBehavior::CODE.skip_kpop_on_initial_pass,
        );
        assert_ne!(
            GateLoopBehavior::DELIGHT.skip_kpop_on_initial_pass,
            GateLoopBehavior::TIDY.skip_kpop_on_initial_pass,
        );
        assert_eq!(GateLoopBehavior::DELIGHT.exit, GateKpopExitPolicy::InitDiscovery);
        assert_eq!(
            GateLoopBehavior::DELIGHT.consecutive_kpop_solved_to_exit(),
            GateLoopBehavior::INIT.consecutive_kpop_solved_to_exit(),
        );
    }
}
