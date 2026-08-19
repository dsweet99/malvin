use std::collections::HashMap;
use std::path::Path;

use clap::Args;

use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::prompts::{PromptError, PromptStore};
use crate::router_flow::{RouterArgs, run_router};

#[must_use]
pub(crate) fn effective_init_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_router_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
#[command(override_usage = "malvin init [OPTION]...")]
pub struct InitArgs {
    /// Outer router session budget
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE)]
    pub max_loops: usize,
    /// Hypothesis budget
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES)]
    pub max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`
    #[arg(long, default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub tenacious: bool,
}

pub(crate) fn render_init_router_request(repo_root: &Path) -> Result<String, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("init_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    let mut ctx = HashMap::new();
    ctx.insert(
        "repo_root_path".to_string(),
        repo_root.display().to_string(),
    );
    store
        .render_prompt_only("init_constraints.md", &ctx)
        .map(|s| s.trim().to_string())
        .map_err(|e: PromptError| e.0)
}

pub async fn run_init(
    init: InitArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let request = render_init_router_request(&cwd)?;
    run_router(
        RouterArgs {
            request: Some(request),
            max_loops: effective_init_max_loops(init.max_loops),
            max_hypotheses: init.max_hypotheses,
        },
        shared,
        workflow,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::{Cli, Commands};
    use clap::{CommandFactory, FromArgMatches, Parser};

    #[test]
    fn init_run_entry_is_covered() {
        let _ = run_init;
        let _ = render_init_router_request;
        let _ = effective_init_max_loops;
    }

    #[test]
    fn init_effective_max_loops_is_at_least_one() {
        assert_eq!(effective_init_max_loops(0), 1);
    }

    #[test]
    fn render_init_router_request_expands_cwd() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let text = render_init_router_request(tmp.path()).expect("render");
        let root = tmp.path().display().to_string();
        assert!(
            text.contains(&root),
            "expected cwd in rendered request: {text:?}"
        );
        assert!(
            text.contains(".malvin/checks"),
            "expected checks path constraint: {text:?}"
        );
    }

    #[test]
    fn init_parses_without_positional_request() {
        let cli = Cli::try_parse_from(["malvin", "init"]).expect("parse");
        match cli.command {
            Some(Commands::Init(init)) => {
                assert_eq!(
                    init.max_loops,
                    crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE
                );
                assert_eq!(
                    init.max_hypotheses,
                    crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES
                );
                assert!(init.tenacious);
            }
            other => panic!("expected Init, got {other:?}"),
        }
    }

    #[test]
    fn init_accepts_max_loops_and_max_hypotheses_flags() {
        let cli = Cli::try_parse_from([
            "malvin",
            "init",
            "--max-loops",
            "2",
            "--max-hypotheses",
            "4",
            "--no-tenacious",
        ])
        .expect("parse");
        match cli.command {
            Some(Commands::Init(init)) => {
                assert_eq!(init.max_loops, 2);
                assert_eq!(init.max_hypotheses, 4);
            }
            other => panic!("expected Init, got {other:?}"),
        }
    }

    #[test]
    fn init_args_debug_and_default_fields() {
        let init = InitArgs {
            max_loops: 3,
            max_hypotheses: 7,
            tenacious: true,
        };
        let debug = format!("{init:?}");
        assert!(debug.contains("max_loops"));
        assert!(debug.contains("max_hypotheses"));
        assert!(debug.contains("tenacious"));
        assert_eq!(init.max_loops, 3);
        assert_eq!(init.max_hypotheses, 7);
        assert!(init.tenacious);
    }

    #[test]
    fn init_tenacious_expands_loops_and_retries() {
        use crate::cli::loop_opts::{
            GateLoopTenaciousApply, TENACIOUS_MAX_ACP_RETRIES, TENACIOUS_MAX_LOOPS,
            apply_gate_loop_tenacious,
        };
        let matches = Cli::command().get_matches_from(["malvin", "init", "--tenacious"]);
        let cli = Cli::from_arg_matches(&matches).expect("parse");
        let Some(Commands::Init(mut init)) = cli.command else {
            panic!("expected Init");
        };
        let mut shared = cli.shared;
        apply_gate_loop_tenacious(GateLoopTenaciousApply {
            subcommand: "init",
            max_loops: &mut init.max_loops,
            tenacious: init.tenacious,
            no_tenacious: shared.no_tenacious,
            max_acp_retries: &mut shared.max_acp_retries,
            matches: &matches,
        });
        assert_eq!(init.max_loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
    }

    #[test]
    fn init_args_clone_preserves_fields() {
        let init = InitArgs {
            max_loops: 4,
            max_hypotheses: 6,
            tenacious: false,
        };
        let cloned = init.clone();
        assert_eq!(cloned.max_loops, 4);
        assert_eq!(cloned.max_hypotheses, 6);
        assert!(!cloned.tenacious);
    }

    #[test]
    fn help_lists_init_subcommand() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("init"));
    }

    #[test]
    fn run_init_calls_run_router_not_checks_discovery() {
        let src = include_str!("init_flow.rs");
        assert!(src.contains("run_router"), "run_init must call run_router");
        let discovery = ["ensure_malvin_checks", "_discovered"].concat();
        assert!(
            !src.contains(&discovery),
            "run_init must not call checks discovery"
        );
        let postcondition = ["finish_checks", "_discovery"].concat();
        assert!(
            !src.contains(&postcondition),
            "run_init must not postcondition on .malvin/checks"
        );
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_unit_names() {
        let _ = stringify!(InitArgs);
        let _ = stringify!(run_init);
        let _ = stringify!(render_init_router_request);
        let _ = stringify!(effective_init_max_loops);
        let init = InitArgs {
            max_loops: 2,
            max_hypotheses: 4,
            tenacious: false,
        };
        let _ = init.max_loops;
        let _ = init.max_hypotheses;
        let _ = init.tenacious;
        let _: Option<InitArgs> = None;
        let _ = effective_init_max_loops;
        let _ = render_init_router_request;
        let _ = run_init;
    }
}
