use crate::do_flow::do_flow_prompt::{
    build_do_coder_run_with_store, prepare_do_prompt_store,
};
use malvin::config::DEFAULT_CLI_MODEL;
use malvin::flow_prompt_join_test_helpers::{
    flow_test_artifacts, flow_test_artifacts_no_checks,
};
use malvin::prompts::{DO_HEADER_MD, HEADER_MD, PromptStore};

fn mock_do_prompt_store(tmp: &tempfile::TempDir) -> PromptStore {
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(HEADER_MD), "CODING_HDR\n").expect("header");
    std::fs::write(prompt_root.join(DO_HEADER_MD), "DO_HDR\n").expect("do_header");
    PromptStore::with_root(prompt_root)
}

fn prepare_do_prompt_store_loads_default_templates() {
    let store = prepare_do_prompt_store().expect("store");
    assert!(store.validate_exists(HEADER_MD).is_ok());
    assert!(store.validate_exists(DO_HEADER_MD).is_ok());
}

fn build_do_coder_run_succeeds_without_checks_in_non_git_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts_no_checks(&tmp);
    let store = prepare_do_prompt_store().expect("store");
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN",
        malvin::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL),
    );
    assert_eq!(run.combined, "USER_TOKEN");
    assert!(
        !run.combined.contains("Know thyself"),
        "headers are sent at spawn, not in the do work prompt"
    );
    assert!(
        !run.combined.contains("malvin --do"),
        "do_header.md is sent at spawn, not in the do work prompt"
    );
}

fn build_do_coder_run_work_prompt_is_user_only() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = mock_do_prompt_store(&tmp);
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN\n\n",
        malvin::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL),
    );
    assert_eq!(run.combined, "USER_TOKEN");
    let (trace_header, trace_user) = &run.header_user_for_trace;
    assert!(trace_header.is_empty());
    assert_eq!(trace_user, "USER_TOKEN");
    assert!(
        !run.combined.contains("CODING_HDR") && !run.combined.contains("DO_HDR"),
        "spawn binds header.md + do_header.md; work turn is user only"
    );
}

fn build_do_coder_run_default_store_work_prompt_is_user() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_do_prompt_store().expect("store");
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN",
        malvin::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL),
    );
    assert_eq!(run.combined, "USER_TOKEN");
}

#[test]
fn kiss_bundled_cli_do_flow_tests() {
    prepare_do_prompt_store_loads_default_templates();
    build_do_coder_run_succeeds_without_checks_in_non_git_workspace();
    build_do_coder_run_work_prompt_is_user_only();
    build_do_coder_run_default_store_work_prompt_is_user();
}
