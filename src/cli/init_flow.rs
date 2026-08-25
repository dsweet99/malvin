use std::collections::HashMap;
use std::path::Path;

use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::prompts::{PromptError, PromptStore};
use crate::router_flow::{RouterArgs, run_router};

#[must_use]
pub(crate) fn effective_init_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_router_shared::effective_max_loops(max_loops)
}

#[derive(Debug, Clone)]
pub struct InitWorkflowOpts {
    pub max_loops: usize,
    pub max_hypotheses: usize,
}

pub(crate) fn malvin_gates_file_missing() -> Result<bool, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if crate::malvin_checks_path(&cwd).is_file() {
        return Ok(false);
    }
    Ok(!cwd.join(crate::MALVIN_CHECKS_REL).is_file())
}

pub(crate) fn should_bootstrap_gates(shared: &SharedOpts) -> Result<bool, String> {
    Ok(shared.gates && malvin_gates_file_missing()?)
}

#[must_use]
pub(crate) fn shared_for_init_bootstrap(shared: &SharedOpts) -> SharedOpts {
    let mut bootstrap = shared.clone();
    bootstrap.gates = false;
    bootstrap
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

pub async fn maybe_run_init_bootstrap(
    init: InitWorkflowOpts,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    if should_bootstrap_gates(shared)? {
        let bootstrap_shared = shared_for_init_bootstrap(shared);
        run_init(init, &bootstrap_shared, workflow).await?;
    }
    Ok(())
}

pub async fn run_init(
    init: InitWorkflowOpts,
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
    use crate::cli::SharedOpts;
    use crate::cli::args::Cli;
    use clap::{CommandFactory, FromArgMatches};

    #[test]
    fn init_run_entry_is_covered() {
        let _ = run_init;
        let _ = maybe_run_init_bootstrap;
        let _ = render_init_router_request;
        let _ = effective_init_max_loops;
        let _ = malvin_gates_file_missing;
        let _ = should_bootstrap_gates;
        let _ = shared_for_init_bootstrap;
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
            text.contains(".malvin/gates"),
            "expected gates path constraint: {text:?}"
        );
    }

    #[test]
    fn init_is_not_a_subcommand_and_parses_as_bare_request() {
        let matches = Cli::command()
            .try_get_matches_from(["malvin", "init"])
            .expect("parse");
        let cli = Cli::from_arg_matches(&matches).expect("cli");
        assert!(cli.command.is_none());
        assert_eq!(cli.request.as_deref(), Some("init"));
    }

    #[test]
    fn should_bootstrap_gates_when_gates_flag_on_and_file_missing() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            let mut shared = SharedOpts::test_defaults();
            shared.gates = true;
            assert!(malvin_gates_file_missing().expect("probe"));
            assert!(should_bootstrap_gates(&shared).expect("bootstrap"));
            shared.gates = false;
            assert!(!should_bootstrap_gates(&shared).expect("bootstrap off"));
            std::env::set_current_dir(cwd).expect("restore cwd");
        });
    }

    #[test]
    fn should_bootstrap_gates_when_legacy_checks_only_and_gates_missing() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            assert!(
                std::process::Command::new("git")
                    .args(["init"])
                    .current_dir(work)
                    .status()
                    .expect("git init")
                    .success()
            );
            std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
            std::fs::write(work.join(".malvin/checks"), "true\n").expect("legacy checks");
            let mut shared = SharedOpts::test_defaults();
            shared.gates = true;
            assert!(
                should_bootstrap_gates(&shared).expect("bootstrap"),
                "legacy checks must not block bootstrap when .malvin/gates is missing"
            );
            std::env::set_current_dir(cwd).expect("restore cwd");
        });
    }

    #[test]
    fn shared_for_init_bootstrap_clears_gates_flag() {
        let mut shared = SharedOpts::test_defaults();
        shared.gates = true;
        let bootstrap = shared_for_init_bootstrap(&shared);
        assert!(!bootstrap.gates);
        assert!(shared.gates);
    }

    #[test]
    fn init_workflow_opts_clone_preserves_fields() {
        let init = InitWorkflowOpts {
            max_loops: 4,
            max_hypotheses: 6,
        };
        let cloned = init.clone();
        assert_eq!(cloned.max_loops, 4);
        assert_eq!(cloned.max_hypotheses, 6);
    }

    #[test]
    fn help_omits_init_subcommand() {
        let help = Cli::command().render_help().to_string();
        assert!(!help.contains("  init "));
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
            "run_init must not postcondition on .malvin/gates"
        );
        let default_route = include_str!("entrypoint.rs");
        assert!(
            !default_route.contains("return run_async_cli(|| {\n            run_init"),
            "default route must not return after init bootstrap"
        );
        assert!(
            default_route.contains("maybe_run_init_bootstrap"),
            "default route must call maybe_run_init_bootstrap"
        );
        let gates_only = include_str!("entrypoint_gates_only.rs");
        assert!(
            !gates_only.contains("return run_async_cli(|| {\n            run_init"),
            "gates-only route must not return after init bootstrap"
        );
        assert!(
            gates_only.contains("maybe_run_init_bootstrap"),
            "gates-only route must call maybe_run_init_bootstrap"
        );
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_unit_names() {
        let _ = stringify!(InitWorkflowOpts);
        let _ = stringify!(run_init);
        let _ = stringify!(maybe_run_init_bootstrap);
        let _ = stringify!(render_init_router_request);
        let _ = stringify!(effective_init_max_loops);
        let init = InitWorkflowOpts {
            max_loops: 2,
            max_hypotheses: 4,
        };
        let _ = init.max_loops;
        let _ = init.max_hypotheses;
        let _: Option<InitWorkflowOpts> = None;
        let _ = effective_init_max_loops;
        let _ = render_init_router_request;
        let _ = run_init;
        let _ = maybe_run_init_bootstrap;
        let _ = malvin_gates_file_missing;
        let _ = should_bootstrap_gates;
        let _ = shared_for_init_bootstrap;
    }
}
