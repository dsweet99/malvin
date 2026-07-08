use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts, user_request_path,
};
use crate::kpop_engine::KPopEnginePrepared;

use super::prep::{priors_kpop_request, priors_preflight, prepare_priors_kpop_prompt_store};

pub struct PriorsKpopPrepared {
    pub inner: KPopEnginePrepared,
    pub resolved_out_path: std::path::PathBuf,
}

fn priors_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<crate::prompt_stratification::WorkflowRenderContext, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(artifacts, "priors")
}

pub fn prepare_priors_kpop_run(
    request: &str,
    out_path: &str,
    workflow: crate::cli::WorkflowCliOptions,
) -> Result<PriorsKpopPrepared, String> {
    let (request_text, resolved_out_path, work_dir) = priors_preflight(request, out_path)?;
    let store = prepare_priors_kpop_prompt_store(workflow)?;
    let artifacts =
        create_kpop_run_artifacts("priors", Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let user_request_disk = user_request_path(&artifacts);
    std::fs::write(&user_request_disk, &request_text).map_err(|e| e.to_string())?;
    let composed = priors_kpop_request(&store, &artifacts, &resolved_out_path, &user_request_disk)?;
    std::fs::write(&artifacts.plan_path, &composed).map_err(|e| e.to_string())?;
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let mut context = priors_kpop_workflow_context(&artifacts)?;
    context.insert(
        "user_request_path".to_string(),
        crate::workflow_context::format_prompt_path(&user_request_disk, &artifacts.work_dir),
    );
    let inner = KPopEnginePrepared {
        artifacts,
        context,
        request_text: composed.clone(),
        startup_emit_request: request.to_string(),
        store,
        malvin_checks_backup,
    };
    Ok(PriorsKpopPrepared {
        inner,
        resolved_out_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_priors_run_startup() {
        let _ = priors_kpop_workflow_context;
        let _ = prepare_priors_kpop_run;
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("priors.md");
        std::fs::write(&path, "body\n").expect("write");
        let store = crate::prompts::PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("priors", Some(tmp.path())).expect("artifacts");
        let prepared = PriorsKpopPrepared {
            inner: KPopEnginePrepared {
                artifacts,
                context: crate::prompt_stratification::WorkflowRenderContext::default(),
                request_text: "req".into(),
                startup_emit_request: "req".into(),
                store,
                malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
            },
            resolved_out_path: path.clone(),
        };
        assert_eq!(prepared.resolved_out_path, path);
        assert_eq!(prepared.inner.request_text, "req");
    }

    #[test]
    fn prepare_priors_kpop_run_succeeds_without_checks() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            std::fs::write(work.join("priors.md"), "body\n").expect("write");
            let prepared = prepare_priors_kpop_run(
                "ground this request",
                "priors.md",
                crate::cli::WorkflowCliOptions { force: true },
            )
            .expect("prepare without checks");
            assert!(!prepared.inner.context.contains_key("quality_gates"));
            assert!(
                prepared.resolved_out_path.ends_with("priors_1.md"),
                "expected sibling allocation, got {}",
                prepared.resolved_out_path.display()
            );
            let user_req = user_request_path(&prepared.inner.artifacts);
            assert!(user_req.is_file(), "must write user_request.md");
            std::env::set_current_dir(cwd).expect("restore");
        });
    }

    #[test]
    fn priors_preflight_allocates_sibling_before_run_dir_created() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            std::fs::write(work.join("priors.md"), "existing\n").expect("write");
            let logs_root = crate::workspace_paths::malvin_logs_root(work);
            let runs_before = crate::log_gc::list_run_dirs(&logs_root).len();
            let prepared = prepare_priors_kpop_run(
                "request text",
                "priors.md",
                crate::cli::WorkflowCliOptions { force: true },
            )
            .expect("default collision must allocate sibling");
            assert!(
                prepared.resolved_out_path.ends_with("priors_1.md"),
                "expected priors_1.md, got {}",
                prepared.resolved_out_path.display()
            );
            let runs_after = crate::log_gc::list_run_dirs(&logs_root).len();
            assert_eq!(
                runs_before + 1,
                runs_after,
                "prepare creates run dir after successful preflight"
            );
            std::env::set_current_dir(cwd).expect("restore");
        });
    }
}
