use crate::prompts::{PromptError, PromptStore};

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};

pub fn prepare_code_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("code_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn code_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<String, String> {
    let mut context =
        crate::orchestrator::workflow_context_paths_only(artifacts, "code");
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
    );
    store
        .render_prompt_only("code_constraints.md", &context)
        .map(|s| s.trim().to_string())
        .map_err(|e: PromptError| e.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_kpop_request_has_no_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "ship widgets\n").expect("write plan");
        let artifacts =
            crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = code_kpop_request(&store, &artifacts).expect("request");
        assert!(
            !text.contains("{{"),
            "code kpop request must expand all placeholders: {text:?}"
        );
        assert!(
            text.contains("plan.md"),
            "expected plan_path in code_constraints request: {text:?}"
        );
    }

    #[test]
    fn prepare_code_kpop_prompt_store_loads_constraints() {
        let workflow = crate::cli::WorkflowCliOptions {
            force: false,
            run_learn: false,
        };
        let store = prepare_code_kpop_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("code_constraints.md").is_ok());
    }
}
