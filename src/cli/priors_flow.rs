use clap::Args;

#[path = "priors_flow/prep.rs"]
mod prep;
#[path = "priors_flow/run_startup.rs"]
mod run_startup;
#[path = "priors_flow/run_loop.rs"]
mod run_loop;

pub use run_loop::run_priors;

#[must_use]
pub(crate) fn effective_priors_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_kpop_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
pub struct PriorsArgs {
    /// Existing `.md` path or literal text describing the request to ground with priors.
    pub request: Option<String>,
    /// Workspace path for the generated priors report (default `priors.md` auto-allocates siblings when occupied).
    #[arg(long, default_value = "priors.md")]
    pub out_path: String,
    /// Maximum gate-loop iterations before stopping.
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE)]
    pub max_loops: usize,
    /// Number of hypotheses per `KPop` round.
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES)]
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
    fn priors_args_default_out_path_is_priors_md() {
        let cli = Cli::try_parse_from(["malvin", "priors", "topic"]).expect("parse");
        match cli.command {
            Some(Commands::Priors(p)) => assert_eq!(p.out_path, "priors.md"),
            other => panic!("expected Priors, got {other:?}"),
        }
    }

    #[test]
    fn priors_out_path_flag_overrides_default() {
        let cli =
            Cli::try_parse_from(["malvin", "priors", "topic", "--out-path", "reports/x.md"]).expect("parse");
        match cli.command {
            Some(Commands::Priors(p)) => assert_eq!(p.out_path, "reports/x.md"),
            other => panic!("expected Priors, got {other:?}"),
        }
    }

    #[test]
    fn priors_out_path_accepts_equals_form() {
        let cli =
            Cli::try_parse_from(["malvin", "priors", "topic", "--out-path=reports/x.md"]).expect("parse");
        match cli.command {
            Some(Commands::Priors(p)) => assert_eq!(p.out_path, "reports/x.md"),
            other => panic!("expected Priors, got {other:?}"),
        }
    }

    #[test]
    fn priors_accepts_optional_request_positional() {
        let cli = Cli::try_parse_from(["malvin", "priors", "focus on CLI UX"]).expect("parse");
        match cli.command {
            Some(Commands::Priors(p)) => assert_eq!(p.request.as_deref(), Some("focus on CLI UX")),
            other => panic!("expected Priors, got {other:?}"),
        }
    }

    #[test]
    fn priors_request_defaults_to_none() {
        let cli = Cli::try_parse_from(["malvin", "priors"]).expect("parse");
        match cli.command {
            Some(Commands::Priors(p)) => assert!(p.request.is_none()),
            other => panic!("expected Priors, got {other:?}"),
        }
    }

    #[test]
    fn priors_effective_max_loops_is_at_least_one() {
        assert_eq!(effective_priors_max_loops(0), 1);
    }

    #[test]
    fn kiss_cov_priors_gate_helpers() {
        let _ = super::run_loop::validate_priors_output;
        let _ = super::run_startup::prepare_priors_kpop_run;
        let _: Option<super::run_startup::PriorsKpopPrepared> = None;
    }

    #[test]
    fn help_lists_priors_subcommand() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("priors"));
    }

    #[test]
    fn priors_tenacious_expands_loops_and_retries() {
        use crate::cli::loop_opts::{
            apply_gate_loop_tenacious, GateLoopTenaciousApply, TENACIOUS_MAX_ACP_RETRIES,
            TENACIOUS_MAX_LOOPS,
        };
        let matches =
            Cli::command().get_matches_from(["malvin", "priors", "topic", "--tenacious"]);
        let cli = Cli::from_arg_matches(&matches).expect("parse");
        let Some(Commands::Priors(mut priors)) = cli.command else {
            panic!("expected Priors");
        };
        let mut shared = cli.shared;
        apply_gate_loop_tenacious(GateLoopTenaciousApply {
            subcommand: "priors",
            max_loops: &mut priors.max_loops,
            tenacious: priors.tenacious,
            no_tenacious: shared.no_tenacious,
            max_acp_retries: &mut shared.max_acp_retries,
            matches: &matches,
        });
        assert_eq!(priors.max_loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
    }
}
