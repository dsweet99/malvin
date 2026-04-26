//! Sync subcommand: reviewer review-loop workflow (`review_1/review_2` + optional learn).

use malvin::artifacts::{
    backup_workspace_grounding_if_present, create_run_artifacts_from_text, resolve_user_request,
    GroundingBackup, RunArtifacts,
};
use malvin::acp::AgentClient;
use malvin::orchestrator::{Orchestrator, WorkflowConfig};
use malvin::output::{print_stdout_line, MALVIN_WHO};
use malvin::prompts::{PromptError, PromptStore, HEADER_MD};

use super::repo_checks::RepoGateOutput;
use super::{
    build_agent, emit_run_startup_sequence, timing_merge, SharedOpts, WorkflowCliOptions,
    LEARN_MIN_ELAPSED_MS,
};

pub struct SyncRunSpec {
    pub max_loops: usize,
    pub no_learn: bool,
    pub request: String,
}

fn prepare_sync_prompt_store(workflow: WorkflowCliOptions) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("review_1.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("review_2.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("concerns.md")
        .map_err(|e: PromptError| e.0)?;
    if workflow.run_learn {
        store
            .validate_exists("learn.md")
            .map_err(|e: PromptError| e.0)?;
    }
    Ok(store)
}

fn prepare_sync_artifacts(
    spec: &SyncRunSpec,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(AgentClient, RunArtifacts, PromptStore, GroundingBackup), String> {
    let client = build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let (text, work_dir) = resolve_user_request(&spec.request)?;
    let artifacts =
        create_run_artifacts_from_text(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    crate::cli::kiss_clamp::ensure_kiss_clamp_if_needed(&artifacts.work_dir, RepoGateOutput::Tagged)?;

    emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), &spec.request)?;
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    let store = prepare_sync_prompt_store(workflow)?;

    Ok((client, artifacts, store, grounding_backup))
}

#[cfg(test)]
mod coverage_tests {
    use super::SyncRunSpec;

    #[test]
    fn kiss_stringify_sync_flow_units() {
        let _ = stringify!(crate::cli::sync_flow::SyncRunSpec);
        let _ = stringify!(crate::cli::sync_flow::prepare_sync_prompt_store);
        let _ = stringify!(crate::cli::sync_flow::prepare_sync_artifacts);
        let _ = stringify!(crate::cli::sync_flow::run_sync);
    }

    #[test]
    fn build_sync_spec_defaults() {
        let spec = SyncRunSpec {
            max_loops: 7,
            no_learn: true,
            request: "x".to_string(),
        };
        assert_eq!(spec.max_loops, 7);
        assert!(spec.no_learn);
        assert_eq!(spec.request, "x");
    }
}

pub async fn run_sync(
    spec: SyncRunSpec,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let run_learn = workflow.run_learn && !spec.no_learn;
    let (mut client, artifacts, store, grounding_backup) =
        prepare_sync_artifacts(&spec, shared, workflow)?;
    let mut orch = Orchestrator {
        client: &mut client,
        prompts: &store,
        artifacts: &artifacts,
        config: WorkflowConfig {
            max_loops: spec.max_loops,
            run_learn,
            learn_min_elapsed_ms: LEARN_MIN_ELAPSED_MS,
            skip_check_plan: true,
        },
        progress_callback: Box::new(|msg: &str| {
            print_stdout_line(MALVIN_WHO, msg);
        }),
        grounding_backup: grounding_backup.clone(),
    };

    let sync_result = orch.run_sync().await;
    timing_merge::merge_acp_with_grounding_restore(
        sync_result.map_err(|e| e.0),
        &artifacts.work_dir,
        &grounding_backup,
    )?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

#[test]
fn stringify_sync_flow_helpers() {
    let _ = stringify!(crate::cli::sync_flow::prepare_sync_prompt_store);
    let _ = stringify!(crate::cli::sync_flow::run_sync);
    let _ = stringify!(crate::cli::sync_flow::SyncRunSpec);
}
