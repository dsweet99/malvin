use crate::flow_prompt_join_test_helpers::flow_test_artifacts;
use crate::router_flow::router_flow_prompt::{
    build_router_b_prompt, prepare_router_prompt_store,
};
use crate::prompts::ROUTER_B_SIMPLE_MD;

#[test]
fn build_router_b_prompt_includes_code_checks_when_coding_task() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "echo ROUTER_CHECK_LINE\n");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_b_prompt(&store, &artifacts, ROUTER_B_SIMPLE_MD, true).expect("router_b");
    assert!(body.contains("echo ROUTER_CHECK_LINE"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_d_prompt_renders_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = crate::router_flow::router_flow_prompt::build_router_d_prompt(&store, &artifacts)
        .expect("router_d");
    assert!(body.contains("Summarize your work"));
    assert!(!body.contains("{{"));
}
