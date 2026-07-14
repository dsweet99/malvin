use crate::kpop_engine::KPopEnginePrepared;
use crate::workflow_context::PromptModelOpts;

use super::prep::{
    materialize_priors_kpop_prepared, prepare_priors_kpop_prompt_store, priors_preflight,
};

pub struct PriorsKpopPrepared {
    pub inner: KPopEnginePrepared,
    pub resolved_out_path: std::path::PathBuf,
}

pub struct PriorsKpopPrepareOpts<'a> {
    pub request: &'a str,
    pub out_path: &'a str,
    pub workflow: crate::cli::WorkflowCliOptions,
    pub model: &'a str,
    pub git: bool,
}

pub fn prepare_priors_kpop_run(
    opts: PriorsKpopPrepareOpts<'_>,
) -> Result<PriorsKpopPrepared, String> {
    let PriorsKpopPrepareOpts {
        request,
        out_path,
        workflow,
        model,
        git,
    } = opts;
    let preflight = priors_preflight(request, out_path)?;
    let store = prepare_priors_kpop_prompt_store(workflow)?;
    let (inner, resolved_out_path) = materialize_priors_kpop_prepared(
        preflight,
        store,
        request.to_string(),
        PromptModelOpts::new(model, git),
    )?;
    Ok(PriorsKpopPrepared {
        inner,
        resolved_out_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_priors_prepare_opts_destructure() {
        let opts = PriorsKpopPrepareOpts {
            request: "ground this",
            out_path: "priors.md",
            workflow: crate::cli::WorkflowCliOptions { force: false },
            model: crate::config::DEFAULT_CLI_MODEL,
            git: true,
        };
        let PriorsKpopPrepareOpts {
            request,
            out_path,
            workflow,
            model,
            git,
        } = opts;
        assert_eq!(request, "ground this");
        assert_eq!(out_path, "priors.md");
        assert!(!workflow.force);
        assert_eq!(model, crate::config::DEFAULT_CLI_MODEL);
        assert!(git);
    }

    #[test]
    fn kiss_cov_priors_run_startup() {
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
            let prepared = prepare_priors_kpop_run(PriorsKpopPrepareOpts {
                request: "ground this request",
                out_path: "priors.md",
                workflow: crate::cli::WorkflowCliOptions { force: true },
                model: crate::config::DEFAULT_CLI_MODEL,
                git: false,
            })
            .expect("prepare without checks");
            assert!(!prepared.inner.context.contains_key("quality_gates"));
            assert!(
                prepared.resolved_out_path.ends_with("priors_1.md"),
                "expected sibling allocation, got {}",
                prepared.resolved_out_path.display()
            );
            let user_req = crate::artifacts::user_request_path(&prepared.inner.artifacts);
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
            let prepared = prepare_priors_kpop_run(PriorsKpopPrepareOpts {
                request: "request text",
                out_path: "priors.md",
                workflow: crate::cli::WorkflowCliOptions { force: true },
                model: crate::config::DEFAULT_CLI_MODEL,
                git: false,
            })
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
