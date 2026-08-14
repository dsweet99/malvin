
use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::kpop_engine::{
    run_kpop_engine, KPopEngineParams, KPopEnginePrepared, KPopHardConstraints,
};
use crate::malvin_config_file::{self, AgentConfig};
use crate::prompts::{PromptError, PromptStore};

use crate::cli::workflow_kpop_shared::{
    kpop_engine_loop_iterations, kpop_workflow_context_without_gates,
};
use crate::cli::{prepare_kpop_prompt_store, SharedOpts, WorkflowCliOptions};

pub(super) fn prepare_checks_discovery_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, true)?;
    store
        .validate_exists("init_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub(super) fn checks_discovery_kpop_request(
    store: &PromptStore,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    let mut ctx = HashMap::new();
    ctx.insert(
        "repo_root_path".to_string(),
        artifacts.work_dir.display().to_string(),
    );
    store
        .render_prompt_only("init_constraints.md", &ctx)
        .map(|s| s.trim().to_string())
        .map_err(|e: PromptError| e.0)
}

pub(super) fn load_discovery_agent_config(work_dir: &Path) -> AgentConfig {
    malvin_config_file::load_malvin_config(work_dir).agent
}

pub(super) async fn run_checks_discovery_kpop(
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
    kpop_command: &str,
) -> Result<(), String> {
    let workflow = WorkflowCliOptions {
        force: !shared.no_force,
    };
    let store = prepare_checks_discovery_prompt_store(workflow)?;
    let request_text = checks_discovery_kpop_request(&store, artifacts)?;
    std::fs::write(&artifacts.plan_path, &request_text).map_err(|e| e.to_string())?;
    let context = kpop_workflow_context_without_gates(artifacts, &shared.model.canonical(), shared.git)?;
    let prepared = KPopEnginePrepared {
        artifacts: artifacts.clone(),
        context,
        request_text,
        store,
    };
    let agent_cfg = load_discovery_agent_config(&artifacts.work_dir);
    let max_loops = if crate::acp::test_no_real_agent_enabled() {
        1
    } else {
        agent_cfg.max_loops
    };
    let _iterations = kpop_engine_loop_iterations(max_loops);
    let (_gates_ok, _agent_ran, _timing, _last_backups) = run_kpop_engine(KPopEngineParams {
        command: kpop_command,
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        max_hypotheses: agent_cfg.max_hypotheses,
        behavior: KPopHardConstraints::CHECKS_DISCOVERY,
    })
    .await?;
    Ok(())
}
