use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts,
};
use crate::cli::cli_request::require_cli_request;
use crate::gate_kpop_workflow::GateKpopPrepared;

use super::prep::{
    explain_kpop_request, explain_preflight, prepare_explain_kpop_prompt_store, ExplainKpopRequestInput,
};

pub struct ExplainKpopPrepared {
    pub inner: GateKpopPrepared,
    pub tex_path: std::path::PathBuf,
    pub pdf_path: std::path::PathBuf,
    pub request_work_dir: std::path::PathBuf,
    pub auto_out_path: bool,
    pub preflight_snapshot: super::prep::ExplainPreflightSnapshot,
}

fn explain_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<std::collections::HashMap<String, String>, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(artifacts, "explain")
}

pub fn prepare_explain_kpop_run(
    request: Option<&String>,
    out_path: &str,
    out_path_explicit: bool,
    workflow: crate::cli::WorkflowCliOptions,
) -> Result<ExplainKpopPrepared, String> {
    let request_arg = require_cli_request(request, "explain")?;
    let (request_text, request_work_dir, outputs, preflight_snapshot) =
        explain_preflight(&request_arg, out_path, out_path_explicit)?;
    let artifact_work_dir = if out_path_explicit {
        crate::artifacts::work_dir_for_path(&outputs.tex_path)
    } else {
        request_work_dir.clone()
    };
    let store = prepare_explain_kpop_prompt_store(workflow)?;
    let artifacts = create_kpop_run_artifacts("explain", Some(artifact_work_dir.as_path()))
        .map_err(|e| e.to_string())?;
    let request_body = explain_kpop_request(
        &store,
        &artifacts,
        ExplainKpopRequestInput {
            request_text: &request_text,
            request_work_dir: &request_work_dir,
            outputs: &outputs,
            out_path_explicit,
        },
    )?;
    std::fs::write(&artifacts.plan_path, &request_body).map_err(|e| e.to_string())?;
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = explain_kpop_workflow_context(&artifacts)?;
    let inner = GateKpopPrepared {
        artifacts,
        context,
        request_text: request_body.clone(),
        startup_emit_request: request_arg,
        store,
        malvin_checks_backup,
    };
    Ok(ExplainKpopPrepared {
        inner,
        tex_path: outputs.tex_path,
        pdf_path: outputs.pdf_path,
        request_work_dir,
        auto_out_path: !out_path_explicit,
        preflight_snapshot,
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
        let _: Option<ExplainKpopPrepared> = None;
    }
}
