use crate::config::DEFAULT_CLI_MODEL;
use crate::flow_prompt_join_test_helpers::flow_test_artifacts;
use crate::router_flow::router_flow_prompt::{
    build_router_kpop_group_prompt, build_router_summarize_prompt, build_router_work_prompt,
    prepare_router_prompt_store, RouterKpopGroupPromptInput, RouterSummarizePromptInput,
    RouterWorkPromptInput,
};
use crate::prompts::PromptStore;

#[test]
fn build_router_work_prompt_expands_malvin_command_with_active_model() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir prompts");
    std::fs::write(
        prompt_root.join("router_work.md"),
        "Use {{ malvin_command }} kpop\n{{ code_extra }}\n",
    )
    .expect("write router_work");
    std::fs::write(
        prompt_root.join("router_code_extra.md"),
        "- Make sure the code checks all pass:\n```\n{{ code_checks }}\n```\n",
    )
    .expect("write code_extra");
    let store = PromptStore::with_root(prompt_root);
    let artifacts = flow_test_artifacts(&tmp);
    let body = build_router_work_prompt(RouterWorkPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: "composer-2",
        git: false,
        gates: false,
    })
    .expect("router_work");
    assert!(body.contains("malvin --model=composer-2"));
    assert!(!body.contains("{{ malvin_command }}"));
}

#[test]
fn build_router_work_prompt_renders_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_work_prompt(RouterWorkPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: false,
    })
    .expect("router_work");
    assert!(!body.contains("CONTINUE_ROUTER"));
    assert!(!body.contains("{{"));
    assert!(
        crate::artifacts::review_requirements_json(&artifacts)
            .file_name()
            .is_some_and(|n| n == "review_requirements.json")
    );
}

#[test]
fn build_router_work_prompt_includes_code_checks_when_gates_enabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "echo ROUTER_CHECK_LINE\n");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_work_prompt(RouterWorkPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: true,
    })
    .expect("router_work");
    assert!(body.contains("echo ROUTER_CHECK_LINE"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_work_prompt_omits_code_checks_when_gates_disabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "echo ROUTER_CHECK_LINE\n");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_work_prompt(RouterWorkPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: false,
    })
    .expect("router_work");
    assert!(!body.contains("echo ROUTER_CHECK_LINE"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_kpop_group_prompt_expands_review_keys_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let exp = artifacts.gate_exp_log_path(1);
    let body = build_router_kpop_group_prompt(RouterKpopGroupPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        groups_block: "### Group 1\nTitle: Checks\n\nRequirements:\n\n- gates pass",
        max_hypotheses: crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
        exp_log: &exp,
    })
    .expect("kpop group");
    assert!(!body.contains("{{"));
    assert!(body.contains("gates pass"));
    assert!(body.contains(&format!("{}", crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES)));
    assert!(
        body.contains("alone on its own line"),
        "router KPop must require isolated heading lines: {body}"
    );
    assert!(
        body.contains("experiment log only"),
        "router KPop must keep summary/tl;dr out of chat: {body}"
    );
}

#[test]
fn build_router_summarize_prompt_renders_dm_body_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_summarize_prompt(RouterSummarizePromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
    })
    .expect("router_summarize");
    assert!(!body.contains("{{"));
    assert!(
        body.contains("Write a summarize of this entire session"),
        "must render router_summarize.md: {body}"
    );
}
