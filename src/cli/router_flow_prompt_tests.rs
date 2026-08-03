use crate::config::DEFAULT_CLI_MODEL;
use crate::flow_prompt_join_test_helpers::flow_test_artifacts;
use crate::router_flow::router_flow_prompt::{
    build_router_a_prompt, build_router_kpop_common_prompt, build_router_summarize_prompt,
    prepare_router_prompt_store, RouterAPromptInput, RouterKpopCommonPromptInput,
    RouterSummarizePromptInput,
};
use crate::prompts::PromptStore;

#[test]
fn build_router_a_prompt_expands_malvin_command_with_active_model() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir prompts");
    std::fs::write(
        prompt_root.join("router_a.md"),
        "Use {{ malvin_command }}\n{{ code_extra }}\nSee {{ user_request_path }}.\n",
    )
    .expect("write router_a");
    std::fs::write(
        prompt_root.join("router_code_extra.md"),
        "- Make sure the code checks all pass:\n```\n{{ code_checks }}\n```\n",
    )
    .expect("write code_extra");
    let store = PromptStore::with_root(prompt_root);
    let artifacts = flow_test_artifacts(&tmp);
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: "composer-2",
        git: false,
        gates: false,
    })
    .expect("router_a");
    assert!(body.contains("malvin --model=composer-2"));
    assert!(!body.contains("{{ malvin_command }}"));
}

#[test]
fn build_router_a_prompt_renders_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: false,
    })
    .expect("router_a");
    assert!(!body.contains("{{"));
    assert!(body.contains("__MALVIN_DONE__"));
}

#[test]
fn build_router_a_prompt_includes_code_checks_when_gates_enabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "echo ROUTER_CHECK_LINE\n");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: true,
    })
    .expect("router_a");
    assert!(body.contains("echo ROUTER_CHECK_LINE"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_a_prompt_omits_code_checks_when_gates_disabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "echo ROUTER_CHECK_LINE\n");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: false,
    })
    .expect("router_a");
    assert!(!body.contains("echo ROUTER_CHECK_LINE"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_kpop_common_prompt_expands_keys_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let exp = artifacts.gate_exp_log_path(1);
    let body = build_router_kpop_common_prompt(RouterKpopCommonPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        max_hypotheses: crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
        exp_log: &exp,
    })
    .expect("kpop common");
    assert!(!body.contains("{{"));
    assert!(body.contains(&format!("{}", crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES)));
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
        body.contains("Write a summary of this entire session"),
        "must render router_summarize.md: {body}"
    );
}
