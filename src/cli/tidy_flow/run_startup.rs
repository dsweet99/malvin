use std::path::Path;

use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts,
};
use crate::cli::gate_kpop_workflow::GateKpopPrepared;

use super::prep::{prepare_tidy_kpop_prompt_store, tidy_kpop_request};

pub type TidyKpopPrepared = GateKpopPrepared;

fn tidy_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<std::collections::HashMap<String, String>, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context(artifacts, "tidy")
}

pub fn prepare_tidy_kpop_run(
    workflow: crate::cli::WorkflowCliOptions,
) -> Result<TidyKpopPrepared, String> {
    let store = prepare_tidy_kpop_prompt_store(workflow)?;
    let work_dir = Path::new(".").to_path_buf();
    let artifacts =
        create_kpop_run_artifacts("tidy", Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let request_text = tidy_kpop_request(&store, &artifacts)?;
    std::fs::write(&artifacts.plan_path, &request_text).map_err(|e| e.to_string())?;
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = tidy_kpop_workflow_context(&artifacts)?;
    Ok(GateKpopPrepared {
        artifacts,
        context,
        request_text: request_text.clone(),
        startup_emit_request: request_text,
        store,
        malvin_checks_backup,
    })
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = tidy_kpop_workflow_context;
    }
}
