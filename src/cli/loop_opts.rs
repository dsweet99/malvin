//! Shared outer-loop CLI helpers (`--tenacious`, effective iteration counts).

use clap::ArgMatches;

use super::config_defaults::global_flag_from_command_line;
use super::config_loop::subcommand_flag_from_command_line;

pub const DEFAULT_TENACIOUS: bool = true;
pub const TENACIOUS_MAX_LOOPS: usize = 9999;
pub const TENACIOUS_MAX_ACP_RETRIES: u32 = 9999;

/// Which loop/retry budgets were set explicitly on the command line (skip tenacious expansion).
pub struct TenaciousBudgetGuard {
    pub max_loops_explicit: bool,
    pub max_acp_retries_explicit: bool,
}

#[must_use]
pub fn tenacious_budget_guard(matches: &ArgMatches, subcommand: &str) -> TenaciousBudgetGuard {
    TenaciousBudgetGuard {
        max_loops_explicit: subcommand_flag_from_command_line(matches, subcommand, "max_loops"),
        max_acp_retries_explicit: global_flag_from_command_line(matches, "max_acp_retries"),
    }
}

/// Inputs for expanding gate-loop budgets when tenacious mode is active.
pub struct GateLoopTenaciousApply<'a> {
    pub subcommand: &'a str,
    pub max_loops: &'a mut usize,
    pub tenacious: bool,
    pub no_tenacious: bool,
    pub max_acp_retries: &'a mut u32,
    pub matches: &'a ArgMatches,
}

pub fn apply_gate_loop_tenacious(input: GateLoopTenaciousApply<'_>) {
    apply_tenacious(
        input.max_loops,
        input.max_acp_retries,
        input.tenacious && !input.no_tenacious,
        tenacious_budget_guard(input.matches, input.subcommand),
    );
}

/// When set, expand to very large `--max-loops` and `--max-acp-retries` budgets unless guarded.
#[allow(clippy::missing_const_for_fn)]
pub fn apply_tenacious(
    max_loops: &mut usize,
    max_acp_retries: &mut u32,
    tenacious: bool,
    guard: TenaciousBudgetGuard,
) {
    if tenacious {
        if !guard.max_loops_explicit {
            *max_loops = TENACIOUS_MAX_LOOPS;
        }
        if !guard.max_acp_retries_explicit {
            *max_acp_retries = TENACIOUS_MAX_ACP_RETRIES;
        }
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
        apply_tenacious(
            &mut loops,
            &mut retries,
            true,
            TenaciousBudgetGuard {
                max_loops_explicit: false,
                max_acp_retries_explicit: false,
            },
        );
        assert_eq!(loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(retries, TENACIOUS_MAX_ACP_RETRIES);
    }

    #[test]
    fn apply_tenacious_skips_explicit_max_loops() {
        let mut loops = 3usize;
        let mut retries = 3u32;
        apply_tenacious(
            &mut loops,
            &mut retries,
            true,
            TenaciousBudgetGuard {
                max_loops_explicit: true,
                max_acp_retries_explicit: false,
            },
        );
        assert_eq!(loops, 3);
        assert_eq!(retries, TENACIOUS_MAX_ACP_RETRIES);
    }

    #[test]
    fn apply_gate_loop_tenacious_expands_unless_opted_out() {
        use clap::CommandFactory;
        use crate::cli::Cli;
        let matches = Cli::command().get_matches_from(["malvin", "kpop", "hello"]);
        let mut loops = 1usize;
        let mut retries = 3u32;
        apply_gate_loop_tenacious(GateLoopTenaciousApply {
            subcommand: "kpop",
            max_loops: &mut loops,
            tenacious: true,
            no_tenacious: false,
            max_acp_retries: &mut retries,
            matches: &matches,
        });
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
#[cfg(test)]
#[path = "loop_opts_test.rs"]
mod loop_opts_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<GateLoopTenaciousApply> = None;
        let _: Option<TenaciousBudgetGuard> = None;
    }
}
