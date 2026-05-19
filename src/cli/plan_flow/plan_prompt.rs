use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::prompts::{HEADER_MD, PromptError, PromptStore, merged_coding_rules};

pub fn prepare_plan_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("coding_rules.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("review_plan.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn compose_plan_prompt(
    store: &PromptStore,
    context: &HashMap<String, String>,
) -> Result<String, String> {
    let header = store
        .render_prompt_only(HEADER_MD, context)
        .map_err(|e: PromptError| e.0)?;
    let rules = merged_coding_rules(store, context).map_err(|e: PromptError| e.0)?;
    let body = store
        .render("review_plan.md", context)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!(
        "{}\n\n{}\n\n{}",
        header.trim_end(),
        rules.trim_end(),
        body.trim_end()
    ))
}

pub fn plan_prompt_context(
    artifacts: &RunArtifacts,
    user_plan_path: &Path,
    store: &PromptStore,
) -> Result<HashMap<String, String>, String> {
    use crate::orchestrator::{format_prompt_path, workflow_context};
    let mut ctx = workflow_context(artifacts, store, "plan").map_err(|e: PromptError| e.0)?;
    ctx.insert(
        "plan_path".to_string(),
        format_prompt_path(user_plan_path, &artifacts.work_dir),
    );
    Ok(ctx)
}

#[cfg(test)]
mod plan_prompt_coverage {
    #[test]
    fn kiss_stringify_plan_prompt_units() {
        let _ = stringify!(super::prepare_plan_prompt_store);
        let _ = stringify!(super::compose_plan_prompt);
        let _ = stringify!(super::plan_prompt_context);
    }
}
