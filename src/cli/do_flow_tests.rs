use crate::do_flow::do_flow_prompt::{
    build_do_coder_run_with_store, prepare_do_prompt_store,
};
use malvin::config::DEFAULT_CLI_MODEL;
use malvin::flow_prompt_join_test_helpers::{
    assert_dual_workflow_header_join, flow_test_artifacts, flow_test_artifacts_no_checks,
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

fn build_do_coder_run_cosends_headers_with_user_in_non_git_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts_no_checks(&tmp);
    let store = prepare_do_prompt_store().expect("store");
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN",
        malvin::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL),
    )
    .expect("build");
    assert!(
        run.combined.contains("Know thyself") || run.combined.contains("MALVIN HEADER"),
        "coding header must ride in the co-sent prompt"
    );
    assert!(
        run.combined.contains("malvin --do") || run.combined.contains("do mode"),
        "do_header.md must ride in the co-sent prompt"
    );
    assert!(
        run.combined.contains("USER_TOKEN"),
        "user request must ride in the same host prompt"
    );
    let (trace_header, trace_user) = &run.header_user_for_trace;
    assert!(!trace_header.is_empty());
    assert_eq!(trace_user, "USER_TOKEN");
}

fn build_do_coder_run_joins_mock_headers_then_user() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = mock_do_prompt_store(&tmp);
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN\n\n",
        malvin::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL),
    )
    .expect("build");
    assert_dual_workflow_header_join(&run.combined, "CODING_HDR", "DO_HDR", "USER_TOKEN");
    let (trace_header, trace_user) = &run.header_user_for_trace;
    assert!(trace_header.contains("CODING_HDR") && trace_header.contains("DO_HDR"));
    assert_eq!(trace_user, "USER_TOKEN");
}

fn build_do_coder_run_default_store_includes_user() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_do_prompt_store().expect("store");
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN",
        malvin::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL),
    )
    .expect("build");
    assert!(run.combined.ends_with("USER_TOKEN"));
}

#[test]
fn kiss_bundled_cli_do_flow_tests() {
    prepare_do_prompt_store_loads_default_templates();
    build_do_coder_run_cosends_headers_with_user_in_non_git_workspace();
    build_do_coder_run_joins_mock_headers_then_user();
    build_do_coder_run_default_store_includes_user();
}
