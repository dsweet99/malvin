//! Shared outer-loop CLI helpers (`--tenacious`, effective iteration counts).

pub const TENACIOUS_MAX_LOOPS: usize = 9999;
pub const TENACIOUS_MAX_ACP_RETRIES: u32 = 9999;

/// When set, expand to very large `--max-loops` and `--max-acp-retries` budgets.
pub const fn apply_tenacious(max_loops: &mut usize, max_acp_retries: &mut u32, tenacious: bool) {
    if tenacious {
        *max_loops = TENACIOUS_MAX_LOOPS;
        *max_acp_retries = TENACIOUS_MAX_ACP_RETRIES;
    }
}

/// Experiment-log iteration index for the `agent_loop`th outer kpop agent (1-based).
#[must_use]
pub const fn kpop_agent_loop_exp_iteration(agent_loop: usize, max_loops: usize) -> usize {
    if max_loops <= 1 {
        0
    } else {
        agent_loop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_tenacious_sets_large_budgets() {
        let mut loops = 1usize;
        let mut retries = 3u32;
        apply_tenacious(&mut loops, &mut retries, true);
        assert_eq!(loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(retries, TENACIOUS_MAX_ACP_RETRIES);
    }

    #[test]
    fn kpop_agent_loop_exp_iteration_uses_legacy_path_for_single_run() {
        assert_eq!(kpop_agent_loop_exp_iteration(1, 1), 0);
        assert_eq!(kpop_agent_loop_exp_iteration(1, 3), 1);
        assert_eq!(kpop_agent_loop_exp_iteration(2, 3), 2);
    }
}
