use crate::prompts::{PromptError, PromptStore};

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::kpop_program::render_repo_program;

pub fn prepare_code_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, true)?;
    store
        .validate_exists("kpop_program.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("code_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn code_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
    model: &str,
) -> Result<String, String> {
    let context = crate::orchestrator::workflow_context_paths_only(artifacts, model);
    render_repo_program(
        store,
        "code_constraints.md",
        context.as_map(),
        artifacts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_kpop_request_has_no_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .expect("git init");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "ship widgets\n").expect("write plan");
        let artifacts =
            crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
        crate::seed_malvin_checks(tmp.path(), "true\n");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = code_kpop_request(&store, &artifacts, crate::config::DEFAULT_CLI_MODEL).expect("request");
        assert!(
            !text.contains("{{"),
            "code kpop request must expand all placeholders: {text:?}"
        );
        let plan_name = artifacts
            .plan_path
            .file_name()
            .and_then(|s| s.to_str())
            .expect("plan filename");
        assert!(
            text.contains(plan_name),
            "expected plan_path in code_constraints request: {text:?}"
        );
        assert!(
            text.contains("Satisfy all constraints"),
            "expected kpop_program wrapper: {text:?}"
        );
    }

    #[test]
    fn prepare_code_kpop_prompt_store_loads_program_and_constraints() {
        let workflow = crate::cli::WorkflowCliOptions {
            force: false,
            
        };
        let store = prepare_code_kpop_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("kpop_program.md").is_ok());
        assert!(store.validate_exists("code_constraints.md").is_ok());
    }
}
