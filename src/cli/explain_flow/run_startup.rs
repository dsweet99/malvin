use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts,
};
use crate::cli::cli_request::require_cli_request;
use crate::cli::gate_kpop_workflow::GateKpopPrepared;

use super::prep::{explain_kpop_request, explain_preflight, prepare_explain_kpop_prompt_store};

pub struct ExplainKpopPrepared {
    pub inner: GateKpopPrepared,
    pub tex_path: std::path::PathBuf,
    pub pdf_path: std::path::PathBuf,
}

fn explain_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<std::collections::HashMap<String, String>, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(artifacts, "explain")
}

pub fn prepare_explain_kpop_run(
    request: Option<&String>,
    out_path: &str,
    workflow: crate::cli::WorkflowCliOptions,
) -> Result<ExplainKpopPrepared, String> {
    let request_arg = require_cli_request(request, "explain")?;
    let (request_text, outputs) = explain_preflight(&request_arg, out_path)?;
    let artifact_work_dir = crate::artifacts::work_dir_for_path(&outputs.tex_path);
    let store = prepare_explain_kpop_prompt_store(workflow)?;
    let artifacts = create_kpop_run_artifacts("explain", Some(artifact_work_dir.as_path()))
        .map_err(|e| e.to_string())?;
    let request_body = explain_kpop_request(&store, &artifacts, &request_text, &outputs)?;
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_explain_run_startup() {
        let _ = explain_kpop_workflow_context;
        let _ = prepare_explain_kpop_run;
    }

    #[test]
    fn explain_preflight_runs_before_run_dir_created() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            let logs_root = crate::workspace_paths::malvin_logs_root(work);
            let runs_before = crate::log_gc::list_run_dirs(&logs_root).len();
            let Err(err) = prepare_explain_kpop_run(
                None,
                "explain.tex",
                crate::cli::WorkflowCliOptions { force: true },
            ) else {
                panic!("missing request must fail");
            };
            assert!(err.contains("missing required REQUEST"));
            let runs_after = crate::log_gc::list_run_dirs(&logs_root).len();
            assert_eq!(runs_before, runs_after, "preflight must not create run dirs");
            std::env::set_current_dir(cwd).expect("restore");
        });
    }

    #[test]
    fn explain_preflight_refuses_stale_outputs_before_run_dir_created() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            std::fs::write(work.join("explain.tex"), "STALE\n").expect("write");
            std::fs::write(work.join("explain.pdf"), b"%PDF").expect("write");
            let logs_root = crate::workspace_paths::malvin_logs_root(work);
            let runs_before = crate::log_gc::list_run_dirs(&logs_root).len();
            let Err(err) = prepare_explain_kpop_run(
                Some(&"topic".to_string()),
                "explain.tex",
                crate::cli::WorkflowCliOptions { force: true },
            ) else {
                panic!("preflight must fail");
            };
            assert!(err.contains("refusing to overwrite"));
            let runs_after = crate::log_gc::list_run_dirs(&logs_root).len();
            assert_eq!(runs_before, runs_after, "preflight must not create run dirs");
            std::env::set_current_dir(cwd).expect("restore");
        });
    }
}
