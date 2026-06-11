use clap::Args;

#[path = "delight_flow/prep.rs"]
mod prep;
#[path = "delight_flow/run_startup.rs"]
mod run_startup;
#[path = "delight_flow/run_loop.rs"]
mod run_loop;

pub use run_loop::run_delight;

#[must_use]
pub(crate) fn effective_delight_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_kpop_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
pub struct DelightArgs {
    /// Workspace path for the generated plan (must not already exist).
    #[arg(long, default_value = "plan.md")]
    pub out_path: String,
    /// Maximum gate-loop iterations before stopping.
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE)]
    pub max_loops: usize,
    /// Number of hypotheses per `KPop` round.
    #[arg(long, default_value_t = 5)]
    pub max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`.
    #[arg(long, default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub tenacious: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::{Cli, Commands};
    use clap::{CommandFactory, FromArgMatches, Parser};

    #[test]
    fn delight_args_default_out_path_is_plan_md() {
        let cli = Cli::try_parse_from(["malvin", "delight"]).expect("parse");
        match cli.command {
            Some(Commands::Delight(d)) => assert_eq!(d.out_path, "plan.md"),
            other => panic!("expected Delight, got {other:?}"),
        }
    }

    #[test]
    fn delight_out_path_flag_overrides_default() {
        let cli = Cli::try_parse_from(["malvin", "delight", "--out-path", "plans/x.md"]).expect("parse");
        match cli.command {
            Some(Commands::Delight(d)) => assert_eq!(d.out_path, "plans/x.md"),
            other => panic!("expected Delight, got {other:?}"),
        }
    }

    #[test]
    fn delight_out_path_accepts_equals_form() {
        let cli = Cli::try_parse_from(["malvin", "delight", "--out-path=plans/x.md"]).expect("parse");
        match cli.command {
            Some(Commands::Delight(d)) => assert_eq!(d.out_path, "plans/x.md"),
            other => panic!("expected Delight, got {other:?}"),
        }
    }

    #[test]
    fn delight_rejects_extra_positional() {
        assert!(Cli::try_parse_from(["malvin", "delight", "extra"]).is_err());
    }

    #[test]
    fn delight_effective_max_loops_is_at_least_one() {
        assert_eq!(effective_delight_max_loops(0), 1);
    }

    #[test]
    fn kiss_cov_delight_gate_helpers() {
    }

    #[test]
    fn help_lists_delight_subcommand() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("delight"));
    }

    #[test]
    fn delight_tenacious_expands_loops_and_retries() {
        use crate::cli::loop_opts::{
            apply_gate_loop_tenacious, GateLoopTenaciousApply, TENACIOUS_MAX_ACP_RETRIES,
            TENACIOUS_MAX_LOOPS,
        };
        let matches = Cli::command().get_matches_from(["malvin", "delight", "--tenacious"]);
        let cli = Cli::from_arg_matches(&matches).expect("parse");
        let Some(Commands::Delight(mut delight)) = cli.command else {
            panic!("expected Delight");
        };
        let mut shared = cli.shared;
        apply_gate_loop_tenacious(GateLoopTenaciousApply {
            subcommand: "delight",
            max_loops: &mut delight.max_loops,
            tenacious: delight.tenacious,
            no_tenacious: shared.no_tenacious,
            max_acp_retries: &mut shared.max_acp_retries,
            matches: &matches,
        });
        assert_eq!(delight.max_loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
    }
}
