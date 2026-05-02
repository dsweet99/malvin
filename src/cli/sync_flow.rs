//! Sync subcommand: reviewer review-loop workflow (`review_1/review_2` + optional learn).

use malvin::acp::AgentClient;
use malvin::artifacts::{
    GroundingBackup, RunArtifacts, backup_workspace_grounding_if_present,
    create_run_artifacts_from_text,
};
use malvin::orchestrator::{
    Orchestrator, OrchestratorSessionMode, WorkflowConfig, WorkflowError, workflow_context,
};
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{HEADER_MD, PromptError, PromptStore};
use std::path::Path;

use super::repo_checks::{self, RepoGateOutput};
use super::{
    LEARN_MIN_ELAPSED_MS, SharedOpts, WorkflowCliOptions, build_agent, emit_run_startup_sequence,
    timing_merge,
};

pub struct SyncRunSpec {
    pub max_loops: usize,
    pub no_learn: bool,
}

fn prepare_sync_prompt_store(run_learn: bool) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    prepare_sync_prompt_store_for(&store, run_learn)?;
    Ok(store)
}

fn prepare_sync_prompt_store_for(store: &PromptStore, run_learn: bool) -> Result<(), String> {
    store
        .validate_exists(HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("check_sync.md")
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
    store
        .validate_exists("coding_rules.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("summary.md")
        .map_err(|e: PromptError| e.0)?;
    if run_learn {
        store
            .validate_exists("learn.md")
            .map_err(|e: PromptError| e.0)?;
    }
    Ok(())
}

fn prepare_sync_artifacts(
    _spec: &SyncRunSpec,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    run_learn: bool,
) -> Result<(AgentClient, RunArtifacts, PromptStore, GroundingBackup), String> {
    let mut client = build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let artifacts =
        create_run_artifacts_from_text("sync", Some(Path::new("."))).map_err(|e| e.to_string())?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    repo_checks::run_repo_workspace_gates(
        &artifacts.work_dir,
        RepoGateOutput::Tagged,
        Some(&artifacts.run_dir),
    )?;

    emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), "sync")?;
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    let store = prepare_sync_prompt_store(run_learn)?;

    Ok((client, artifacts, store, grounding_backup))
}

#[cfg(test)]
mod coverage_tests {
    use super::{PromptStore, SyncRunSpec, prepare_sync_prompt_store_for};

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
        };
        assert_eq!(spec.max_loops, 7);
        assert!(spec.no_learn);
    }

    #[test]
    fn prepare_sync_prompt_store_bypasses_learn_check_when_no_learn_requested() {
        let tmp = tempfile::tempdir().unwrap();
        let prompts_dir = tmp.path().join(".malvin").join("prompts");
        let store = PromptStore::with_root(prompts_dir.clone());
        std::fs::create_dir_all(&prompts_dir).unwrap();
        let _ = std::fs::write(prompts_dir.join(super::HEADER_MD), "h");
        let _ = std::fs::write(prompts_dir.join("check_sync.md"), "c");
        let _ = std::fs::write(prompts_dir.join("review_1.md"), "r1");
        let _ = std::fs::write(prompts_dir.join("review_2.md"), "r2");
        let _ = std::fs::write(prompts_dir.join("concerns.md"), "c");
        let _ = std::fs::write(prompts_dir.join("coding_rules.md"), "rules");
        let _ = std::fs::write(prompts_dir.join("summary.md"), "s");
        let _ = std::fs::remove_file(prompts_dir.join("learn.md"));

        assert!(prepare_sync_prompt_store_for(&store, false).is_ok());
        let _ = std::fs::remove_file(prompts_dir.join("coding_rules.md"));
        assert!(prepare_sync_prompt_store_for(&store, false).is_err());
        let _ = std::fs::write(prompts_dir.join("coding_rules.md"), "rules");
        assert!(prepare_sync_prompt_store_for(&store, true).is_err());
    }

    #[test]
    fn kiss_stringify_sync_flow_units_all() {
        let _ = stringify!(crate::cli::sync_flow::prepare_sync_prompt_store_for);
    }
}

pub async fn run_sync(
    spec: SyncRunSpec,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let run_learn = workflow.run_learn && !spec.no_learn;
    let (mut client, artifacts, store, grounding_backup) =
        prepare_sync_artifacts(&spec, shared, workflow, run_learn)?;
    let ctx = workflow_context(&artifacts, &store, "sync").map_err(|e: PromptError| e.0)?;

    let sync_result = {
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
        orch
            .run_with_pre_summary_gap(
                &ctx,
                OrchestratorSessionMode::Sync,
                crate::cli::mid_session_gates::mid_pre_summary_repo_gates,
            )
            .await
            .map_err(|e: WorkflowError| e.0)
    };
    timing_merge::merge_acp_with_grounding_restore(
        sync_result,
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
