use std::collections::HashMap;
use std::path::Path;

use crate::prompts::{PromptError, PromptStore};

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};

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

pub fn tidy_kpop_request(store: &PromptStore, work_dir: &Path) -> Result<String, String> {
    let scope_constraints = store
        .render_prompt_only("tidy_constraints.md", &HashMap::new())
        .map_err(|e: PromptError| e.0)?;
    let quality_gates =
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(work_dir)?;
    let mut context = HashMap::new();
    context.insert(
        "scope_constraints".to_string(),
        scope_constraints.trim().to_string(),
    );
    context.insert("quality_gates".to_string(), quality_gates);
    store
        .render_prompt_only("kpop_program.md", &context)
        .map(|s| s.trim().to_string())
        .map_err(|e: PromptError| e.0)
}

pub fn write_checks_do_not_pass_to_review_path(review_path: &Path) -> Result<(), String> {
    if let Some(parent) = review_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create parent dirs for {}: {e}",
                review_path.display()
            )
        })?;
    }
    std::fs::write(review_path, b"Checks do not pass\n").map_err(|e| {
        format!(
            "failed to write checks-do-not-pass marker {}: {e}",
            review_path.display()
        )
    })
}

pub fn write_checks_do_not_pass_for_artifacts(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<(), String> {
    write_checks_do_not_pass_to_review_path(&artifacts.artifact_review_md())?;
    write_checks_do_not_pass_to_review_path(&artifacts.workspace_review_md())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tidy_kpop_request_has_no_unresolved_braces() {
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = tidy_kpop_request(&store, Path::new(".")).expect("request");
        assert!(
            !text.contains("{{"),
            "tidy kpop request must expand all placeholders: {text:?}"
        );
        assert!(
            text.contains("Just get quality gates to pass"),
            "expected tidy_constraints in request: {text:?}"
        );
    }

    #[test]
    fn prepare_tidy_kpop_prompt_store_loads_program_and_constraints() {
        let workflow = crate::cli::WorkflowCliOptions {
            force: false,
            run_learn: false,
        };
        let store = prepare_tidy_kpop_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("kpop_program.md").is_ok());
        assert!(store.validate_exists("tidy_constraints.md").is_ok());
    }
}
