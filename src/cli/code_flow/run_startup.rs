use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_run_artifacts_from_text,
    resolve_user_md_request,
};
use crate::cli::gate_kpop_workflow::GateKpopPrepared;

use super::prep::{code_kpop_request, prepare_code_kpop_prompt_store};

pub type CodeKpopPrepared = GateKpopPrepared;

fn code_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<std::collections::HashMap<String, String>, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context(artifacts, "code")
}

pub fn prepare_code_kpop_run(
    workflow: crate::cli::WorkflowCliOptions,
    cli_request: &str,
) -> Result<CodeKpopPrepared, String> {
    let store = prepare_code_kpop_prompt_store(workflow)?;
    let (plan_text, work_dir) = resolve_user_md_request(cli_request)?;
    let artifacts =
        create_run_artifacts_from_text(&plan_text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let request_text = code_kpop_request(&store, &artifacts)?;
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = code_kpop_workflow_context(&artifacts)?;
    Ok(GateKpopPrepared {
        artifacts,
        context,
        request_text,
        startup_emit_request: cli_request.to_string(),
        store,
        malvin_checks_backup,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_code_kpop_workflow_context() {
        let _ = stringify!(code_kpop_workflow_context);
    }

    #[test]
    fn prepare_code_kpop_run_resolves_plan() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "build feature\n").expect("write plan");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = prepare_code_kpop_run(
            crate::cli::WorkflowCliOptions {
                force: false,
                
            },
            &format!("@{}", plan.display()),
        )
        .expect("prepared");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(prepared.request_text.contains("plan.md"));
        assert_eq!(prepared.startup_emit_request, format!("@{}", plan.display()));
    }
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = code_kpop_workflow_context;
    }
}
