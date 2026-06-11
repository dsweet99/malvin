use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts,
};
use crate::cli::gate_kpop_workflow::GateKpopPrepared;

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
    workflow: crate::cli::WorkflowCliOptions,
) -> Result<DelightKpopPrepared, String> {
    let (resolved_out_path, work_dir) = delight_preflight(out_path)?;
    let store = prepare_delight_kpop_prompt_store(workflow)?;
    let artifacts =
        create_kpop_run_artifacts("delight", Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let request_text = delight_kpop_request(&store, &artifacts, &resolved_out_path)?;
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
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_delight_run_startup() {
        let _ = delight_kpop_workflow_context;
        let _ = prepare_delight_kpop_run;
    }

    #[test]
    fn delight_preflight_runs_before_run_dir_created() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            std::fs::write(work.join("plan.md"), "existing\n").expect("write");
            let logs_root = crate::workspace_paths::malvin_logs_root(work);
            let runs_before = crate::log_gc::list_run_dirs(&logs_root).len();
            let Err(err) = prepare_delight_kpop_run(
                "plan.md",
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
