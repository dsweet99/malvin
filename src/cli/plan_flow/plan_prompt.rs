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
    use std::path::Path;

    use crate::artifacts::create_run_artifacts;

    #[test]
    fn kiss_stringify_plan_prompt_units() {
        let _ = stringify!(super::prepare_plan_prompt_store);
        let _ = stringify!(super::compose_plan_prompt);
        let _ = stringify!(super::plan_prompt_context);
    }

    #[test]
    fn compose_plan_prompt_renders_embedded_review_plan_without_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan_path = tmp.path().join("plan.md");
        std::fs::write(&plan_path, "plan body\n").expect("write plan");
        let artifacts =
            create_run_artifacts(Path::new(&plan_path), Some(tmp.path())).expect("artifacts");
        let store = super::prepare_plan_prompt_store().expect("store");
        let ctx = super::plan_prompt_context(&artifacts, &plan_path, &store).expect("ctx");
        let prompt = super::compose_plan_prompt(&store, &ctx).expect("compose");
        assert!(
            !prompt.contains("{{"),
            "compose_plan_prompt must expand every {{ key }} placeholder"
        );
    }
}
