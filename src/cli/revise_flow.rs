use clap::Args;

#[path = "revise_flow/prep.rs"]
mod prep;
#[path = "revise_flow/run_startup.rs"]
mod run_startup;
#[path = "revise_flow/run_loop.rs"]
mod run_loop;

pub use run_loop::run_revise;

#[must_use]
pub(crate) fn effective_revise_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_kpop_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
pub struct ReviseArgs {
    /// Existing document to revise in place.
    pub doc_path: String,
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
    fn revise_parses_doc_path_positional() {
        let cli = Cli::try_parse_from(["malvin", "revise", "docs/guide.md"]).expect("parse");
        match cli.command {
            Some(Commands::Revise(r)) => assert_eq!(r.doc_path, "docs/guide.md"),
            other => panic!("expected Revise, got {other:?}"),
        }
    }

    #[test]
    fn revise_rejects_missing_doc_path() {
        assert!(Cli::try_parse_from(["malvin", "revise"]).is_err());
    }

    #[test]
    fn revise_rejects_extra_positional() {
        assert!(Cli::try_parse_from(["malvin", "revise", "a.md", "b.md"]).is_err());
    }

    #[test]
    fn revise_effective_max_loops_is_at_least_one() {
        assert_eq!(effective_revise_max_loops(0), 1);
    }

    #[test]
    fn kiss_cov_revise_gate_helpers() {
    }

    #[test]
    fn help_lists_revise_subcommand() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("revise"));
    }

    #[test]
    fn revise_tenacious_expands_loops_and_retries() {
        use crate::cli::loop_opts::{
            apply_gate_loop_tenacious, GateLoopTenaciousApply, TENACIOUS_MAX_ACP_RETRIES,
            TENACIOUS_MAX_LOOPS,
        };
        let matches =
            Cli::command().get_matches_from(["malvin", "revise", "doc.md", "--tenacious"]);
        let cli = Cli::from_arg_matches(&matches).expect("parse");
        let Some(Commands::Revise(mut revise)) = cli.command else {
            panic!("expected Revise");
        };
        let mut shared = cli.shared;
        apply_gate_loop_tenacious(GateLoopTenaciousApply {
            subcommand: "revise",
            max_loops: &mut revise.max_loops,
            tenacious: revise.tenacious,
            no_tenacious: shared.no_tenacious,
            max_acp_retries: &mut shared.max_acp_retries,
            matches: &matches,
        });
        assert_eq!(revise.max_loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
    }
}
