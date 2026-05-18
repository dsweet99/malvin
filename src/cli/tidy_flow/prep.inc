use std::collections::HashMap;
use std::path::Path;

use malvin::prompts::{HEADER_MD, PromptError, PromptStore, merged_coding_rules};

pub fn prepare_tidy_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("tidy.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("coding_rules.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("summary.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("reviewers_spawn.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("review_write.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("tidy_concerns.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn compose_tidy_prompt(
    store: &PromptStore,
    context: &HashMap<String, String>,
) -> Result<String, String> {
    let header = store
        .render_prompt_only(HEADER_MD, context)
        .map_err(|e: PromptError| e.0)?;
    let rules = merged_coding_rules(store, context).map_err(|e: PromptError| e.0)?;
    let tidy = store
        .render("tidy.md", context)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!(
        "{}\n\n{}\n\n{}",
        header.trim_end(),
        rules.trim_end(),
        tidy.trim_end()
    ))
}

pub fn compose_tidy_concerns_prompt(
    store: &PromptStore,
    context: &HashMap<String, String>,
) -> Result<String, String> {
    store
        .render("tidy_concerns.md", context)
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
    artifacts: &malvin::artifacts::RunArtifacts,
) -> Result<(), String> {
    write_checks_do_not_pass_to_review_path(&artifacts.artifact_review_md())?;
    write_checks_do_not_pass_to_review_path(&artifacts.workspace_review_md())
}
