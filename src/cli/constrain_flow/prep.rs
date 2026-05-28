use crate::prompts::{PromptError, PromptStore};

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::cli::workflow_kpop_shared::render_kpop_program_request;

pub fn prepare_constrain_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("kpop_program.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("constrain_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn constrain_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<String, String> {
    let mut context =
        crate::orchestrator::workflow_context_paths_only(artifacts, "constrain");
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
    );
    render_kpop_program_request(
        store,
        "constrain_constraints.md",
        &context,
        artifacts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constrain_kpop_request_has_no_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "ship widgets\n").expect("write plan");
        let artifacts =
            crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = constrain_kpop_request(&store, &artifacts).expect("request");
        assert!(
            !text.contains("{{"),
            "constrain kpop request must expand all placeholders: {text:?}"
        );
        assert!(
            text.contains("plan.md"),
            "expected plan_path in constrain_constraints request: {text:?}"
        );
        assert!(
            text.contains("regression test"),
            "expected constrain_constraints in request: {text:?}"
        );
        assert!(
            text.contains("Satisfy all constraints"),
            "expected kpop_program wrapper: {text:?}"
        );
    }

    #[test]
    fn prepare_constrain_kpop_prompt_store_loads_program_and_constraints() {
        let workflow = crate::cli::WorkflowCliOptions {
            force: false,
        };
        let store = prepare_constrain_kpop_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("kpop_program.md").is_ok());
        assert!(store.validate_exists("constrain_constraints.md").is_ok());
    }
}
