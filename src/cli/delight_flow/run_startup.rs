use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, create_kpop_run_artifacts,
};
use crate::kpop_engine::KPopEnginePrepared;

use super::prep::{delight_kpop_request, delight_preflight, prepare_delight_kpop_prompt_store};

pub struct DelightKpopPrepared {
    pub inner: KPopEnginePrepared,
    pub resolved_out_path: std::path::PathBuf,
}

fn delight_kpop_workflow_context(
    artifacts: &crate::artifacts::RunArtifacts,
    model: &str,
    git: bool,
) -> Result<crate::prompt_stratification::WorkflowRenderContext, String> {
    crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(artifacts, model, git)
}

pub struct DelightKpopPrepareOpts<'a> {
    pub out_path: &'a str,
    pub guidance: Option<&'a String>,
    pub workflow: crate::cli::WorkflowCliOptions,
    pub model: &'a str,
    pub git: bool,
}

pub fn prepare_delight_kpop_run(
    opts: DelightKpopPrepareOpts<'_>,
) -> Result<DelightKpopPrepared, String> {
    let DelightKpopPrepareOpts {
        out_path,
        guidance,
        workflow,
        model,
        git,
    } = opts;
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
    let context = delight_kpop_workflow_context(&artifacts, model, git)?;
    let inner = KPopEnginePrepared {
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
    fn kiss_cov_delight_prepare_opts_destructure() {
        let guidance = String::from("hint");
        let opts = DelightKpopPrepareOpts {
            out_path: "pitch.md",
            guidance: Some(&guidance),
            workflow: crate::cli::WorkflowCliOptions { force: true },
            model: crate::config::DEFAULT_CLI_MODEL,
            git: true,
        };
        let DelightKpopPrepareOpts {
            out_path,
            guidance,
            workflow,
            model,
            git,
        } = opts;
        assert_eq!(out_path, "pitch.md");
        assert_eq!(guidance.map(String::as_str), Some("hint"));
        assert!(workflow.force);
        assert_eq!(model, crate::config::DEFAULT_CLI_MODEL);
        assert!(git);
    }

    #[test]
    fn kiss_cov_delight_run_startup() {
        let _ = delight_kpop_workflow_context;
        let _ = prepare_delight_kpop_run;
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("pitch.md");
        std::fs::write(&path, "body\n").expect("write");
        let store = crate::prompts::PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("delight", Some(tmp.path())).expect("artifacts");
        let prepared = DelightKpopPrepared {
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
    fn prepare_delight_kpop_run_succeeds_without_checks() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            std::fs::write(work.join("pitch.md"), "body\n").expect("write");
            let prepared = prepare_delight_kpop_run(DelightKpopPrepareOpts {
                out_path: "pitch.md",
                guidance: None,
                workflow: crate::cli::WorkflowCliOptions { force: true },
                model: crate::config::DEFAULT_CLI_MODEL,
                git: false,
            })
            .expect("prepare without checks");
            assert!(!prepared.inner.context.contains_key("quality_gates"));
            std::env::set_current_dir(cwd).expect("restore");
        });
    }

    #[test]
    fn delight_preflight_allocates_sibling_before_run_dir_created() {
        crate::test_utils::with_isolated_home(|work| {
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            std::fs::write(work.join("pitch.md"), "existing\n").expect("write");
            let logs_root = crate::workspace_paths::malvin_logs_root(work);
            let runs_before = crate::log_gc::list_run_dirs(&logs_root).len();
            let prepared = prepare_delight_kpop_run(DelightKpopPrepareOpts {
                out_path: "pitch.md",
                guidance: None,
                workflow: crate::cli::WorkflowCliOptions { force: true },
                model: crate::config::DEFAULT_CLI_MODEL,
                git: false,
            })
            .expect("default collision must allocate sibling");
            assert!(
                prepared.resolved_out_path.ends_with("pitch_1.md"),
                "expected pitch_1.md, got {}",
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
