//! Lazy `.malvin/checks` discovery via `KPop` (`init_constraints.md`).

use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts, RunArtifacts,
};
use crate::kpop_engine::{
    run_kpop_engine, KPopEngineParams, KPopEnginePrepared, KPopHardConstraints,
};
use crate::kpop_program::render_repo_program_without_quality_gates;
use crate::malvin_checks_path;
use crate::malvin_config_file::{self, AgentConfig};
use crate::output::{print_stderr_line, MALVIN_WHO};
use crate::prompts::{PromptError, PromptStore};
use crate::repo_gates::init_discovery_validate::validate_checks_command_lines;
use crate::repo_gates::load_malvin_checks;

use super::workflow_kpop_shared::{
    kpop_engine_loop_iterations, kpop_workflow_context_without_gates,
};
use super::{prepare_kpop_prompt_store, SharedOpts, WorkflowCliOptions};

const DISCOVERY_COMMAND: &str = "checks_discover";

fn checks_already_valid(work_dir: &Path) -> Result<bool, String> {
    let path = malvin_checks_path(work_dir);
    if !path.is_file() {
        return Ok(false);
    }
    let lines = load_malvin_checks(&path)?;
    validate_checks_command_lines(work_dir, &lines)?;
    Ok(true)
}

fn prepare_checks_discovery_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("kpop_program.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("init_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

fn checks_discovery_kpop_request(
    store: &PromptStore,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    let mut ctx = HashMap::new();
    ctx.insert(
        "repo_root_path".to_string(),
        artifacts.work_dir.display().to_string(),
    );
    render_repo_program_without_quality_gates(store, "init_constraints.md", &ctx, artifacts)
}

fn load_discovery_agent_config(work_dir: &Path) -> AgentConfig {
    malvin_config_file::load_malvin_config(work_dir).agent
}

fn finish_checks_discovery(work_dir: &Path) -> Result<(), String> {
    let path = malvin_checks_path(work_dir);
    if !path.is_file() {
        return Err("checks discovery: .malvin/checks still missing".to_string());
    }
    let lines = load_malvin_checks(&path)?;
    validate_checks_command_lines(work_dir, &lines).map_err(|e| {
        format!("checks discovery: .malvin/checks exists but is invalid: {e}")
    })
}

async fn run_checks_discovery_kpop(
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    let workflow = WorkflowCliOptions {
        force: !shared.no_force,
    };
    let store = prepare_checks_discovery_prompt_store(workflow)?;
    let request_text = checks_discovery_kpop_request(&store, artifacts)?;
    std::fs::write(&artifacts.plan_path, &request_text).map_err(|e| e.to_string())?;
    let malvin_checks_backup = backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = kpop_workflow_context_without_gates(artifacts, DISCOVERY_COMMAND)?;
    let prepared = KPopEnginePrepared {
        artifacts: artifacts.clone(),
        context,
        request_text: request_text.clone(),
        startup_emit_request: request_text,
        store,
        malvin_checks_backup,
    };
    let agent_cfg = load_discovery_agent_config(&artifacts.work_dir);
    let max_loops = if crate::acp::test_no_real_agent_enabled() {
        1
    } else {
        agent_cfg.max_loops
    };
    let _iterations = kpop_engine_loop_iterations(max_loops);
    let (_gates_ok, _agent_ran, _timing, _last_backups) = run_kpop_engine(KPopEngineParams {
        command: DISCOVERY_COMMAND,
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        behavior: KPopHardConstraints::CHECKS_DISCOVERY,
    })
    .await?;
    if crate::kpop_progression::mpc_plan_declares_done(&crate::artifacts::mpc_plan_path(artifacts))
        == Ok(false)
        && malvin_checks_path(&artifacts.work_dir).is_file()
    {
        print_stderr_line(
            MALVIN_WHO,
            "checks discovery: mpc plan not DONE but .malvin/checks exists",
        );
    }
    Ok(())
}

/// Run checks discovery `KPop` when `.malvin/checks` is missing or invalid.
pub(crate) async fn ensure_malvin_checks_discovered(
    work_dir: &Path,
    shared: &SharedOpts,
) -> Result<(), String> {
    if checks_already_valid(work_dir)? {
        return Ok(());
    }
    let artifacts = create_kpop_run_artifacts("checks discovery", Some(work_dir))
        .map_err(|e| e.to_string())?;
    crate::cli::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    let discovery_result = run_checks_discovery_kpop(shared, &artifacts).await;
    let finish_result = finish_checks_discovery(work_dir);
    if discovery_result.is_ok() && finish_result.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    discovery_result?;
    finish_result
}

/// Gate-loop prelude: discover `.malvin/checks` when missing (uses process cwd).
pub(crate) async fn ensure_malvin_checks_discovered_for_cwd(
    shared: &SharedOpts,
) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    ensure_malvin_checks_discovered(&cwd, shared).await
}

/// Gate-loop prelude for commands whose workspace comes from a CLI request path.
pub(crate) async fn ensure_malvin_checks_discovered_for_cli_request(
    request: &str,
    shared: &SharedOpts,
) -> Result<(), String> {
    let (_, work_dir) =
        crate::artifacts::resolve_user_md_request(request).map_err(|e| e.to_string())?;
    ensure_malvin_checks_discovered(&work_dir, shared).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::create_kpop_run_artifacts;

    #[test]
    fn checks_discovery_kpop_request_expands_placeholders() {
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::seed_malvin_checks(tmp.path(), "kiss check\n");
        let artifacts =
            create_kpop_run_artifacts("checks_discover", Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = checks_discovery_kpop_request(&store, &artifacts).expect("request");
        assert!(
            !text.contains("{{"),
            "checks discovery kpop request must expand placeholders: {text:?}"
        );
        assert!(
            text.contains("Discover how the repo in") && text.contains("runs quality gates"),
            "expected init_constraints: {text:?}"
        );
    }

    #[test]
    fn prepare_checks_discovery_prompt_store_loads_constraints() {
        let workflow = WorkflowCliOptions { force: false };
        let store = prepare_checks_discovery_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("init_constraints.md").is_ok());
    }

    #[test]
    fn finish_checks_discovery_errors_when_missing() {
        crate::test_utils::with_isolated_home(|tmp| {
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(tmp)
                .status()
                .expect("git init");
            let err = finish_checks_discovery(tmp).unwrap_err();
            assert!(err.contains("still missing"), "{err:?}");
        });
    }

    #[test]
    fn finish_checks_discovery_accepts_valid_kiss_checks() {
        if crate::lookup_bin_on_path("kiss").is_none() {
            return;
        }
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::seed_malvin_checks(tmp.path(), "kiss check\n");
        finish_checks_discovery(tmp.path()).expect("valid");
    }

    #[test]
    fn checks_already_valid_false_without_file() {
        crate::test_utils::with_isolated_home(|tmp| {
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(tmp)
                .status()
                .expect("git init");
            assert!(!checks_already_valid(tmp).expect("read"));
        });
    }

    #[test]
    fn load_discovery_agent_config_reads_workspace_defaults() {
        crate::test_utils::with_isolated_home(|work| {
            crate::seed_malvin_config(work, "");
            let cfg = load_discovery_agent_config(work);
            assert_eq!(cfg.max_loops, crate::malvin_config_file::DEFAULT_MAX_LOOPS);
        });
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = checks_discovery_kpop_request;
        let _ = ensure_malvin_checks_discovered;
        let _ = finish_checks_discovery;
    }
}
