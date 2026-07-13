use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts,
};
use crate::kpop_engine::KPopEnginePrepared;

use super::prep::{prepare_revise_kpop_prompt_store, revise_kpop_request, revise_preflight};

pub struct ReviseKpopPrepared {
    pub inner: KPopEnginePrepared,
    pub resolved_doc_path: std::path::PathBuf,
}

fn revise_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
    model: &str,
) -> Result<crate::prompt_stratification::WorkflowRenderContext, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(artifacts, model)
}

pub fn prepare_revise_kpop_run(
    doc_path: &str,
    workflow: crate::cli::WorkflowCliOptions,
    model: &str,
) -> Result<ReviseKpopPrepared, String> {
    let (resolved_doc_path, work_dir) = revise_preflight(doc_path)?;
    let store = prepare_revise_kpop_prompt_store(workflow)?;
    let artifacts =
        create_kpop_run_artifacts("revise", Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let request_text = revise_kpop_request(&store, &artifacts, &resolved_doc_path)?;
    std::fs::write(&artifacts.plan_path, &request_text).map_err(|e| e.to_string())?;
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = revise_kpop_workflow_context(&artifacts, model)?;
    let inner = KPopEnginePrepared {
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
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("doc.md");
        std::fs::write(&path, "body\n").expect("write");
        let store = crate::prompts::PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("revise", Some(tmp.path())).expect("artifacts");
        let prepared = ReviseKpopPrepared {
            inner: KPopEnginePrepared {
                artifacts,
                context: crate::prompt_stratification::WorkflowRenderContext::default(),
                request_text: "req".into(),
                startup_emit_request: "req".into(),
                store,
                malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
            },
            resolved_doc_path: path.clone(),
        };
        assert_eq!(prepared.resolved_doc_path, path);
        assert_eq!(prepared.inner.request_text, "req");
    }

    #[test]
    fn prepare_revise_kpop_run_succeeds_without_checks() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            std::fs::write(work.join("doc.md"), "body\n").expect("write");
            let prepared = prepare_revise_kpop_run(
                "doc.md",
                crate::cli::WorkflowCliOptions { force: true },
                crate::config::DEFAULT_CLI_MODEL,
            )
            .expect("prepare without checks");
            assert!(!prepared.inner.context.contains_key("quality_gates"));
            std::env::set_current_dir(cwd).expect("restore");
        });
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
                crate::config::DEFAULT_CLI_MODEL,
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
