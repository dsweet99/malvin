use std::path::Path;

use crate::artifacts::{
    MalvinChecksBackup, RunArtifacts, backup_workspace_malvin_checks_if_present,
    create_kpop_run_artifacts,
};
use crate::prompts::PromptStore;
use super::prep::{prepare_tidy_kpop_prompt_store, tidy_kpop_request};

pub struct TidyKpopPrepared {
    pub artifacts: RunArtifacts,
    pub exp_log_path: std::path::PathBuf,
    pub context: std::collections::HashMap<String, String>,
    pub request_text: String,
    pub store: PromptStore,
    pub malvin_checks_backup: MalvinChecksBackup,
}

fn tidy_kpop_workflow_context(
    artifacts: &RunArtifacts,
) -> Result<std::collections::HashMap<String, String>, String> {
    let mut context = crate::orchestrator::workflow_context_paths_only(artifacts, "tidy");
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
    );
    Ok(context)
}

pub fn prepare_tidy_kpop_run(
    workflow: crate::cli::WorkflowCliOptions,
) -> Result<TidyKpopPrepared, String> {
    let store = prepare_tidy_kpop_prompt_store(workflow)?;
    let work_dir = Path::new(".").to_path_buf();
    let artifacts =
        create_kpop_run_artifacts("tidy", Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let request_text = tidy_kpop_request(&store, &work_dir, &artifacts)?;
    std::fs::write(&artifacts.plan_path, &request_text).map_err(|e| e.to_string())?;
    let exp_log_path = artifacts.exp_log_path();
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = tidy_kpop_workflow_context(&artifacts)?;
    Ok(TidyKpopPrepared {
        artifacts,
        exp_log_path,
        context,
        request_text,
        store,
        malvin_checks_backup,
    })
}
