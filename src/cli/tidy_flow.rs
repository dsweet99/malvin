use clap::Args;

#[path = "tidy_flow/run.rs"]
mod run;

pub use run::run_tidy;

#[cfg(test)]
pub(crate) use run::{TIDY_ROUTER_REQUEST, tidy_shared_with_gates_forced};

#[must_use]
pub(crate) fn effective_tidy_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_router_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
#[command(override_usage = "malvin tidy [OPTION]...")]
pub struct TidyArgs {
    /// Outer router session budget
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE)]
    pub max_loops: usize,
    /// Hypothesis budget
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES)]
    pub max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`
    #[arg(long, default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub tenacious: bool,
    #[arg(long, default_value_t = false, hide = true)]
    pub quick: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::SharedOpts;
    use crate::cli::args::{Cli, Commands};
    use clap::{CommandFactory, FromArgMatches, Parser};

    #[test]
    fn tidy_effective_max_loops_is_at_least_one() {
        assert_eq!(effective_tidy_max_loops(0), 1);
    }

    #[test]
    fn tidy_router_request_is_get_the_gates_to_pass() {
        assert_eq!(TIDY_ROUTER_REQUEST, "Get the gates to pass.");
    }

    #[test]
    fn tidy_forces_gates_on_regardless_of_cli() {
        let shared = SharedOpts::test_defaults();
        assert!(!shared.gates);
        let forced = tidy_shared_with_gates_forced(&shared);
        assert!(forced.gates);
    }

    #[test]
    fn help_lists_tidy_subcommand() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("tidy"));
    }

    #[test]
    fn tidy_tenacious_expands_loops_and_retries() {
        use crate::cli::loop_opts::{
            apply_gate_loop_tenacious, GateLoopTenaciousApply, TENACIOUS_MAX_ACP_RETRIES,
            TENACIOUS_MAX_LOOPS,
        };
        let matches = Cli::command().get_matches_from(["malvin", "tidy", "--tenacious"]);
        let cli = Cli::from_arg_matches(&matches).expect("parse");
        let Some(Commands::Tidy(mut tidy)) = cli.command else {
            panic!("expected Tidy");
        };
        let mut shared = cli.shared;
        apply_gate_loop_tenacious(GateLoopTenaciousApply {
            subcommand: "tidy",
            max_loops: &mut tidy.max_loops,
            tenacious: tidy.tenacious,
            no_tenacious: shared.no_tenacious,
            max_acp_retries: &mut shared.max_acp_retries,
            matches: &matches,
        });
        assert_eq!(tidy.max_loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
    }

    #[test]
    fn tidy_parses_without_positional_request() {
        let cli = Cli::try_parse_from(["malvin", "tidy"]).expect("parse");
        match cli.command {
            Some(Commands::Tidy(t)) => {
                assert_eq!(t.max_loops, crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE);
                assert_eq!(
                    t.max_hypotheses,
                    crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES
                );
                assert!(t.tenacious);
                assert!(!t.quick);
            }
            other => panic!("expected Tidy, got {other:?}"),
        }
    }

    #[test]
    fn tidy_accepts_max_loops_and_max_hypotheses_flags() {
        let cli = Cli::try_parse_from([
            "malvin",
            "tidy",
            "--max-loops",
            "7",
            "--max-hypotheses",
            "9",
            "--no-tenacious",
        ])
        .expect("parse");
        match cli.command {
            Some(Commands::Tidy(t)) => {
                assert_eq!(t.max_loops, 7);
                assert_eq!(t.max_hypotheses, 9);
            }
            other => panic!("expected Tidy, got {other:?}"),
        }
    }

    #[test]
    fn tidy_args_clone_preserves_fields() {
        let tidy = TidyArgs {
            max_loops: 4,
            max_hypotheses: 6,
            tenacious: false,
            quick: true,
        };
        let cloned = tidy.clone();
        assert_eq!(cloned.max_loops, 4);
        assert_eq!(cloned.max_hypotheses, 6);
        assert!(!cloned.tenacious);
        assert!(cloned.quick);
    }
}
