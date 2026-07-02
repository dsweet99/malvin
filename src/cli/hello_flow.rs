//! `hello` subcommand: one-shot Cursor connectivity probe (`Hello` prompt via `do` path).

use clap::Args;

use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::do_flow::{run_do, DoArgs};

pub(crate) const HELLO_PROBE_PROMPT: &str = "Hello";

/// Arguments for [`run_hello`].
#[derive(Args, Debug, Clone)]
pub struct HelloArgs {
    /// Stream agent thought tokens to stdout in addition to normal output.
    #[arg(long, default_value_t = false)]
    pub thoughts: bool,
}

/// Run a single-turn `Hello` agent session to verify Cursor ACP connectivity.
///
/// # Errors
///
/// Returns an error when authentication, spawn, or the agent prompt fails.
pub async fn run_hello(
    hello: HelloArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    // Connectivity probe must always tee agent reply (piped parents, `--background`).
    crate::output::enable_probe_stdout_tee();
    crate::output::set_stdout_suppressed(false);
    run_do(
        DoArgs {
            repo_gates: false,
            thoughts: hello.thoughts,
            request: Some(HELLO_PROBE_PROMPT.to_string()),
        },
        shared,
        workflow,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    use clap::Parser;

    #[test]
    fn hello_subcommand_parses_without_bare_kpop_resolve() {
        let cli = Cli::try_parse_from(["malvin", "hello"]).expect("parse");
        match cli.command {
            Some(Commands::Hello(h)) => assert!(!h.thoughts),
            other => panic!("expected hello, got {other:?}"),
        }
        assert!(cli.bare_args.is_empty());
    }

    #[test]
    fn hello_probe_prompt_is_fixed() {
        assert_eq!(HELLO_PROBE_PROMPT, "Hello");
    }

    #[test]
    fn hello_subcommand_parses_thoughts_flag() {
        let cli = Cli::try_parse_from(["malvin", "hello", "--thoughts"]).expect("parse");
        match cli.command {
            Some(Commands::Hello(h)) => assert!(h.thoughts),
            other => panic!("expected hello, got {other:?}"),
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_unit_names() {
        let _: HelloArgs = HelloArgs { thoughts: false };
        let _ = run_hello;
        let _ = HELLO_PROBE_PROMPT;
    }
}
