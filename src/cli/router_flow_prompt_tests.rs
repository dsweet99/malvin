use crate::config::DEFAULT_CLI_MODEL;
use crate::flow_prompt_join_test_helpers::flow_test_artifacts;
use crate::router_flow::router_flow_prompt::{
    build_router_b_prompt, build_router_c_prompt, prepare_router_prompt_store, RouterBPromptInput,
};
use crate::prompts::ROUTER_B_SIMPLE_MD;

#[test]
fn build_router_b_prompt_expands_malvin_command_with_active_model() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_b_prompt(RouterBPromptInput {
        store: &store,
        artifacts: &artifacts,
        template: ROUTER_B_SIMPLE_MD,
        coding_task: false,
        model: "composer-2",
        git: false,
    })
    .expect("router_b");
    assert!(body.contains("malvin --model=composer-2"));
    assert!(!body.contains("{{ malvin_command }}"));
}

#[test]
fn build_router_b_prompt_renders_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_b_prompt(RouterBPromptInput {
        store: &store,
        artifacts: &artifacts,
        template: ROUTER_B_SIMPLE_MD,
        coding_task: false,
        model: DEFAULT_CLI_MODEL,
        git: false,
    })
    .expect("router_b");
    assert!(!body.contains("CONTINUE_ROUTER"));
    assert!(!body.contains("{{"));
    let router_c = build_router_c_prompt(&store, &artifacts, DEFAULT_CLI_MODEL, false).expect("router_c");
    assert!(router_c.contains("CONTINUE_ROUTER"));
    assert!(!router_c.contains("{{"));
}

#[test]
fn build_router_b_prompt_includes_code_checks_when_coding_task() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "echo ROUTER_CHECK_LINE\n");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_b_prompt(RouterBPromptInput {
        store: &store,
        artifacts: &artifacts,
        template: ROUTER_B_SIMPLE_MD,
        coding_task: true,
        model: DEFAULT_CLI_MODEL,
        git: false,
    }).expect("router_b");
    assert!(body.contains("echo ROUTER_CHECK_LINE"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_d_prompt_renders_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = crate::router_flow::router_flow_prompt::build_router_d_prompt(&store, &artifacts, DEFAULT_CLI_MODEL, false)
        .expect("router_d");
    assert!(body.contains("Summarize your work"));
    assert!(!body.contains("{{"));
}
