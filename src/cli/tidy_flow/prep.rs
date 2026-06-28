use std::collections::HashMap;

use crate::prompts::{PromptError, PromptStore};

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::kpop_program::render_repo_program;

pub fn prepare_tidy_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("kpop_program.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("tidy_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn tidy_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<String, String> {
    render_repo_program(store, "tidy_constraints.md", &HashMap::new(), artifacts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kpop_program::kpop_program_context;

    #[test]
    fn tidy_kpop_request_has_no_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("tidy", Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = tidy_kpop_request(&store, &artifacts).expect("request");
        assert!(
            !text.contains("{{"),
            "tidy kpop request must expand all placeholders: {text:?}"
        );
        assert!(
            text.contains("Just get quality gates to pass"),
            "expected tidy_constraints in request: {text:?}"
        );
        assert!(
            text.contains("quality_gates.log"),
            "expected quality_gates_path in kpop request: {text:?}"
        );
        assert!(
            text.contains("Satisfy all constraints"),
            "expected kpop_program wrapper: {text:?}"
        );
    }

    #[test]
    fn prepare_tidy_kpop_prompt_store_loads_program_and_constraints() {
        let workflow = crate::cli::WorkflowCliOptions {
            force: false,
            
        };
        let store = prepare_tidy_kpop_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("kpop_program.md").is_ok());
        assert!(store.validate_exists("tidy_constraints.md").is_ok());
    }

    #[test]
    fn kpop_program_context_includes_scope_and_gates() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("tidy", Some(tmp.path())).expect("artifacts");
        let ctx = kpop_program_context(tmp.path(), "scope", &artifacts).expect("context");
        assert_eq!(ctx.get("scope_constraints").map(String::as_str), Some("scope"));
        assert!(ctx.contains_key("quality_gates"));
        assert!(ctx.contains_key("quality_gates_path"));
        assert!(!ctx.contains_key("quality_gates_log"));
    }
}
