use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts,
};
use crate::gate_kpop_workflow::GateKpopPrepared;

use super::prep::{delight_kpop_request, delight_preflight, prepare_delight_kpop_prompt_store};

pub struct DelightKpopPrepared {
    pub inner: GateKpopPrepared,
    pub resolved_out_path: std::path::PathBuf,
}

fn delight_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<std::collections::HashMap<String, String>, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(artifacts, "delight")
}

pub fn prepare_delight_kpop_run(
    out_path: &str,
    guidance: Option<&String>,
    workflow: crate::cli::WorkflowCliOptions,
) -> Result<DelightKpopPrepared, String> {
    let (resolved_out_path, work_dir) = delight_preflight(out_path)?;
    let store = prepare_delight_kpop_prompt_store(workflow)?;
    let artifacts =
        create_kpop_run_artifacts("delight", Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let resolved_guidance = super::prep::resolve_delight_guidance(guidance)?;
    let request_text = delight_kpop_request(
        &store,
        &artifacts,
        &resolved_out_path,
        resolved_guidance.as_deref(),
    )?;
    std::fs::write(&artifacts.plan_path, &request_text).map_err(|e| e.to_string())?;
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = delight_kpop_workflow_context(&artifacts)?;
    let inner = GateKpopPrepared {
        artifacts,
        context,
        request_text: request_text.clone(),
        startup_emit_request: request_text,
        store,
        malvin_checks_backup,
    };
    Ok(DelightKpopPrepared {
        inner,
        resolved_out_path,
    })
}
#[cfg(test)]
#[path = "run_startup_test.rs"]
mod run_startup_test;
#[cfg(test)]
#[path = "run_startup_kiss_cov_test.rs"]
mod run_startup_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<DelightKpopPrepared> = None;
    }
}
