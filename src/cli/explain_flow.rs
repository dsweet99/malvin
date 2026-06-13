use clap::Args;

#[path = "explain_flow/prep.rs"]
mod prep;
#[path = "explain_flow/run_startup.rs"]
mod run_startup;
#[path = "explain_flow/run_loop.rs"]
mod run_loop;

pub use run_loop::run_explain;

pub(crate) use prep::explain_revise_doc_path;

#[must_use]
pub(crate) fn effective_explain_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_kpop_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
pub struct ExplainArgs {
    /// Existing `.md` path or literal text describing what to explain.
    pub request: Option<String>,
    /// Workspace path for the LaTeX output (PDF is the same path with `.pdf`; default basename stays in the request work directory).
    #[arg(long, default_value = "explain.tex")]
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
    fn explain_args_default_out_path_is_explain_tex() {
        let cli = Cli::try_parse_from(["malvin", "explain", "topic"]).expect("parse");
        match cli.command {
            Some(Commands::Explain(e)) => assert_eq!(e.out_path, "explain.tex"),
            other => panic!("expected Explain, got {other:?}"),
        }
    }

    #[test]
    fn explain_out_path_flag_overrides_default() {
        let cli = Cli::try_parse_from(["malvin", "explain", "topic", "--out-path", "docs/paper.tex"]).expect("parse");
        match cli.command {
            Some(Commands::Explain(e)) => assert_eq!(e.out_path, "docs/paper.tex"),
            other => panic!("expected Explain, got {other:?}"),
        }
    }

    #[test]
    fn explain_out_path_accepts_equals_form() {
        let cli = Cli::try_parse_from(["malvin", "explain", "topic", "--out-path=docs/paper.tex"]).expect("parse");
        match cli.command {
            Some(Commands::Explain(e)) => assert_eq!(e.out_path, "docs/paper.tex"),
            other => panic!("expected Explain, got {other:?}"),
        }
    }

    #[test]
    fn explain_parses_request_positional() {
        let cli = Cli::try_parse_from(["malvin", "explain", "how it works"]).expect("parse");
        match cli.command {
            Some(Commands::Explain(e)) => assert_eq!(e.request.as_deref(), Some("how it works")),
            other => panic!("expected Explain, got {other:?}"),
        }
    }

    #[test]
    fn explain_rejects_extra_positional() {
        assert!(Cli::try_parse_from(["malvin", "explain", "a", "b"]).is_err());
    }

    #[test]
    fn explain_effective_max_loops_is_at_least_one() {
        assert_eq!(effective_explain_max_loops(0), 1);
    }

    #[test]
    fn kiss_cov_explain_gate_helpers() {
        let _ = super::run_loop::validate_explain_output;
        let _ = super::run_startup::prepare_explain_kpop_run;
        let _: Option<super::run_startup::ExplainKpopPrepared> = None;
    }

    #[test]
    fn help_lists_explain_subcommand() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("explain"));
    }

    #[test]
    fn explain_tenacious_expands_loops_and_retries() {
        use crate::cli::loop_opts::{
            apply_gate_loop_tenacious, GateLoopTenaciousApply, TENACIOUS_MAX_ACP_RETRIES,
            TENACIOUS_MAX_LOOPS,
        };
        let matches = Cli::command().get_matches_from(["malvin", "explain", "topic", "--tenacious"]);
        let cli = Cli::from_arg_matches(&matches).expect("parse");
        let Some(Commands::Explain(mut explain)) = cli.command else {
            panic!("expected Explain");
        };
        let mut shared = cli.shared;
        apply_gate_loop_tenacious(GateLoopTenaciousApply {
            subcommand: "explain",
            max_loops: &mut explain.max_loops,
            tenacious: explain.tenacious,
            no_tenacious: shared.no_tenacious,
            max_acp_retries: &mut shared.max_acp_retries,
            matches: &matches,
        });
        assert_eq!(explain.max_loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
    }
}
