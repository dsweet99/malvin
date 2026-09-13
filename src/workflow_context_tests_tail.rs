use std::collections::HashMap;

use super::{
    AGENTS_MD_FILENAME, format_agents_md_insert, insert_current_state, insert_formatted,
    resolve_user_brief_path, workflow_context_paths_only,
};
use crate::prompt_stratification::WorkflowRenderContext;

fn format_agents_md_insert_cases() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert_eq!(format_agents_md_insert(tmp.path()), "");
    std::fs::write(tmp.path().join(AGENTS_MD_FILENAME), "   \n").expect("blank");
    assert_eq!(format_agents_md_insert(tmp.path()), "");
    std::fs::write(tmp.path().join(AGENTS_MD_FILENAME), "Use ripwire.\n").expect("body");
    let got = format_agents_md_insert(tmp.path());
    assert!(
        got.contains("## Workspace `AGENTS.md`") && got.contains("Use ripwire."),
        "expected labeled AGENTS.md body, got {got:?}"
    );
}

fn workflow_context_paths_only_embeds_agents_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    std::fs::write(tmp.path().join(AGENTS_MD_FILENAME), "Prefer narrow checks.\n").expect("agents");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = workflow_context_paths_only(&artifacts, crate::config::DEFAULT_CLI_MODEL);
    let insert = ctx.get("agents_insert").expect("agents_insert");
    assert!(
        insert.contains("Prefer narrow checks."),
        "workspace AGENTS.md must populate agents_insert: {insert:?}"
    );
}

fn workflow_context_paths_only_omits_removed_git_extra() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = workflow_context_paths_only(&artifacts, crate::config::DEFAULT_CLI_MODEL);
    assert!(
        ctx.get("git_extra").is_none(),
        "git_extra placeholder must not remain after --git removal"
    );
}

fn insert_current_state_populates_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let mut ctx = HashMap::new();
    insert_current_state(&mut ctx, &artifacts, tmp.path());
    let state = ctx.get("current_state").expect("state");
    assert!(state.contains("User:"));
    assert!(state.contains("Date/time:"));
    assert!(state.contains("Sandbox memory:"));
}

fn insert_formatted_stores_workflow_relative_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "plan_path", &plan, tmp.path());
    assert_eq!(ctx.get("plan_path").map(String::as_str), Some("./plan.md"));
}

fn resolve_user_brief_path_uses_context_override() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let override_path = tmp.path().join("user_request.md");
    std::fs::write(&override_path, "u").expect("write");
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "user_request_path", &override_path, tmp.path());
    let resolved = resolve_user_brief_path(&artifacts, &WorkflowRenderContext::from(ctx));
    assert_eq!(
        resolved,
        override_path.canonicalize().expect("canonicalize")
    );
}

fn resolve_user_brief_path_falls_back_to_plan_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let resolved = resolve_user_brief_path(&artifacts, &WorkflowRenderContext::default());
    assert_eq!(
        resolved,
        artifacts
            .plan_path
            .canonicalize()
            .unwrap_or_else(|_| artifacts.plan_path.clone())
    );
}

fn workflow_context_returns_plan_path_and_quality_gates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan_path = tmp.path().join("plan.md");
    std::fs::write(&plan_path, "plan\n").expect("write plan");
    crate::seed_malvin_checks(tmp.path(), "true\n");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan_path, Some(tmp.path())).expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let ctx = super::workflow_context(&artifacts, &store, crate::config::DEFAULT_CLI_MODEL)
        .expect("context");
    assert!(ctx.contains_key("plan_path"));
    assert!(ctx.contains_key("quality_gates"));
}

#[test]
fn kiss_bundled_workflow_context_tests_tail() {
    format_agents_md_insert_cases();
    workflow_context_paths_only_embeds_agents_md();
    workflow_context_paths_only_omits_removed_git_extra();
    insert_current_state_populates_key();
    insert_formatted_stores_workflow_relative_path();
    resolve_user_brief_path_uses_context_override();
    resolve_user_brief_path_falls_back_to_plan_path();
    workflow_context_returns_plan_path_and_quality_gates();
}
