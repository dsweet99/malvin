use clap::Args;

pub(crate) mod prep;
mod run;

pub use run::run_write;

#[must_use]
pub(crate) fn effective_write_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_kpop_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
pub struct WriteArgs {
    /// Existing `.md` path or literal text describing what to write about.
    pub request: Option<String>,
    /// Workspace path for the LaTeX output (PDF is the same path with `.pdf`; default basename stays in the request work directory).
    #[arg(long, default_value = "write.tex")]
    pub out_path: String,
    /// Outer router session budget (`effective_max_loops`).
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE)]
    pub max_loops: usize,
    /// Hypothesis budget for `kpop_common.md` (`{{ max_hypotheses }}`).
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_WRITE_MAX_HYPOTHESES)]
    pub max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`.
    #[arg(long, default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub tenacious: bool,
    /// Set when the user passes `--out-path` on the command line (not a clap flag).
    #[arg(skip)]
    pub out_path_explicit: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::{Cli, Commands};
    use clap::{CommandFactory, FromArgMatches, Parser};

    #[test]
    fn write_args_default_out_path_is_write_tex() {
        let cli = Cli::try_parse_from(["malvin", "write", "topic"]).expect("parse");
        match cli.command {
            Some(Commands::Write(e)) => assert_eq!(e.out_path, "write.tex"),
            other => panic!("expected Write, got {other:?}"),
        }
    }

    #[test]
    fn write_out_path_flag_overrides_default() {
        let cli = Cli::try_parse_from(["malvin", "write", "topic", "--out-path", "docs/paper.tex"]).expect("parse");
        match cli.command {
            Some(Commands::Write(e)) => assert_eq!(e.out_path, "docs/paper.tex"),
            other => panic!("expected Write, got {other:?}"),
        }
    }

    #[test]
    fn write_out_path_accepts_equals_form() {
        let cli = Cli::try_parse_from(["malvin", "write", "topic", "--out-path=docs/paper.tex"]).expect("parse");
        match cli.command {
            Some(Commands::Write(e)) => assert_eq!(e.out_path, "docs/paper.tex"),
            other => panic!("expected Write, got {other:?}"),
        }
    }

    #[test]
    fn write_parses_request_positional() {
        let cli = Cli::try_parse_from(["malvin", "write", "how it works"]).expect("parse");
        match cli.command {
            Some(Commands::Write(e)) => assert_eq!(e.request.as_deref(), Some("how it works")),
            other => panic!("expected Write, got {other:?}"),
        }
    }

    #[test]
    fn write_rejects_extra_positional() {
        assert!(Cli::try_parse_from(["malvin", "write", "a", "b"]).is_err());
    }

    #[test]
    fn write_effective_max_loops_is_at_least_one() {
        assert_eq!(effective_write_max_loops(0), 1);
    }

    #[test]
    fn help_lists_write_subcommand() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("write"));
    }

    #[test]
    fn write_tenacious_expands_loops_and_retries() {
        use crate::cli::loop_opts::{
            apply_gate_loop_tenacious, GateLoopTenaciousApply, TENACIOUS_MAX_ACP_RETRIES,
            TENACIOUS_MAX_LOOPS,
        };
        let matches = Cli::command().get_matches_from(["malvin", "write", "topic", "--tenacious"]);
        let cli = Cli::from_arg_matches(&matches).expect("parse");
        let Some(Commands::Write(mut write_args)) = cli.command else {
            panic!("expected Write");
        };
        let mut shared = cli.shared;
        apply_gate_loop_tenacious(GateLoopTenaciousApply {
            subcommand: "write",
            max_loops: &mut write_args.max_loops,
            tenacious: write_args.tenacious,
            no_tenacious: shared.no_tenacious,
            max_acp_retries: &mut shared.max_acp_retries,
            matches: &matches,
        });
        assert_eq!(write_args.max_loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
    }
}
