pub(crate) struct GateLoopBehavior {
    pub skip_kpop_on_initial_pass: bool,
    pub recheck_gates_after_exhausted: bool,
}

impl GateLoopBehavior {
    pub const CODE: Self = Self {
        skip_kpop_on_initial_pass: false,
        recheck_gates_after_exhausted: true,
    };
    pub const TIDY: Self = Self {
        skip_kpop_on_initial_pass: true,
        recheck_gates_after_exhausted: false,
    };
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
}
