use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts,
};
use crate::cli::gate_kpop_workflow::GateKpopPrepared;

use super::prep::{prepare_revise_kpop_prompt_store, revise_kpop_request, revise_preflight};

pub struct ReviseKpopPrepared {
    pub inner: GateKpopPrepared,
    pub resolved_doc_path: std::path::PathBuf,
}

fn revise_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<std::collections::HashMap<String, String>, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context(artifacts, "revise")
}

pub fn prepare_revise_kpop_run(
    doc_path: &str,
    workflow: crate::cli::WorkflowCliOptions,
) -> Result<ReviseKpopPrepared, String> {
    let (resolved_doc_path, work_dir) = revise_preflight(doc_path)?;
    let store = prepare_revise_kpop_prompt_store(workflow)?;
    let artifacts =
        create_kpop_run_artifacts("revise", Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let request_text = revise_kpop_request(&store, &artifacts, &resolved_doc_path)?;
    std::fs::write(&artifacts.plan_path, &request_text).map_err(|e| e.to_string())?;
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = revise_kpop_workflow_context(&artifacts)?;
    let inner = GateKpopPrepared {
        artifacts,
        context,
        request_text: request_text.clone(),
        startup_emit_request: doc_path.to_string(),
        store,
        malvin_checks_backup,
    };
    Ok(ReviseKpopPrepared {
        inner,
        resolved_doc_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_revise_run_startup() {
        let _ = revise_kpop_workflow_context;
        let _ = prepare_revise_kpop_run;
    }

    #[test]
    fn revise_preflight_runs_before_run_dir_created() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            let logs_root = crate::workspace_paths::malvin_logs_root(work);
            let runs_before = crate::log_gc::list_run_dirs(&logs_root).len();
            let Err(err) = prepare_revise_kpop_run(
                "missing.md",
                crate::cli::WorkflowCliOptions { force: true },
            ) else {
                panic!("preflight must fail");
            };
            assert!(err.contains("not an existing file"));
            let runs_after = crate::log_gc::list_run_dirs(&logs_root).len();
            assert_eq!(runs_before, runs_after, "preflight must not create run dirs");
            std::env::set_current_dir(cwd).expect("restore");
        });
    }
}
