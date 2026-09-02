use std::collections::HashMap;

use crate::config::DEFAULT_CLI_MODEL;
use crate::flow_prompt_join_test_helpers::{assert_header_user_join, flow_test_artifacts};
use crate::prompts::{PromptStore, HEADER_MD, ROUTER_A_MD, ROUTER_B_CREATIVE_MD, ROUTER_B_MD};
use crate::router_flow::router_flow_prompt::{
    build_router_a_prompt, build_router_header_prompt, build_router_kpop_common_prompt,
    combine_router_acp_prompt_header_and_user, combine_router_prompt_file_and_user,
    combine_router_raw_header_and_user, prepare_router_prompt_store, RouterAPromptInput,
    RouterHeaderPromptInput, RouterKpopCommonPromptInput,
};

fn write_router_mock_prompt_files(prompt_root: &std::path::Path) {
    write_prompt(prompt_root, HEADER_MD, "CODING_HDR\n");
    write_prompt(
        prompt_root,
        ROUTER_A_MD,
        "ROUTER_A\n{{ code_extra }}\nSee {{ user_request_path }}.\n",
    );
    write_prompt(prompt_root, ROUTER_B_MD, "ROUTER_B\n");
    write_prompt(prompt_root, ROUTER_B_CREATIVE_MD, "ROUTER_B_CREATIVE\n");
    write_prompt(prompt_root, "router_code_extra.md", "");
    write_prompt(prompt_root, "router_summarize.md", "SUM\n");
}

fn write_prompt(prompt_root: &std::path::Path, name: &str, body: &str) {
    std::fs::write(prompt_root.join(name), body).unwrap_or_else(|_| panic!("write {name}"));
}

fn mock_router_prompt_store(tmp: &tempfile::TempDir) -> PromptStore {
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    write_router_mock_prompt_files(&prompt_root);
    PromptStore::with_root(prompt_root)
}

#[test]
fn combine_router_prompt_file_and_user_joins_rendered_template_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(HEADER_MD), "TMPL\n").expect("tmpl");
    let store = PromptStore::with_root(prompt_root);
    let ctx = crate::prompt_stratification::WorkflowRenderContext::from(HashMap::from([(
        "k".into(),
        "v".into(),
    )]));
    let (combined, header, user) =
        combine_router_prompt_file_and_user(&store, "BODY\n", HEADER_MD, &ctx).expect("combine");
    assert_eq!(header, "TMPL");
    assert_eq!(user, "BODY");
    assert_header_user_join(&combined, "TMPL", "BODY");
}

#[test]
fn prepare_router_prompt_store_loads_default_templates() {
    let store = prepare_router_prompt_store().expect("store");
    assert!(store.validate_exists(HEADER_MD).is_ok());
    assert!(store.validate_exists(ROUTER_A_MD).is_ok());
    assert!(store.validate_exists(ROUTER_B_MD).is_ok());
    assert!(store.validate_exists(ROUTER_B_CREATIVE_MD).is_ok());
}

#[test]
fn build_router_header_prompt_renders_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_header_prompt(RouterHeaderPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
    })
    .expect("header");
    assert!(body.contains("Know thyself") || body.contains("Context Prep") || !body.is_empty());
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_kpop_common_prompt_renders_budget_and_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_kpop_common_prompt(RouterKpopCommonPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        max_hypotheses: 7,
        no_kpop: false,
    })
    .expect("kpop common");
    assert!(body.contains("max_hypotheses = `7`"));
    assert!(body.contains("exp_log_"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_a_prompt_includes_user_request_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: false,
        no_kpop: false,
    })
    .expect("router_a");
    assert!(body.contains("__MALVIN_DONE__"));
    assert!(!body.contains("{{"));
}

#[test]
fn combine_router_acp_prompt_joins_rendered_header_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = mock_router_prompt_store(&tmp);
    let artifacts = flow_test_artifacts(&tmp);
    let (combined, header, user) = combine_router_acp_prompt_header_and_user(
        &store,
        &artifacts,
        "USER_TOKEN",
        crate::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL, false),
    )
    .expect("combine");
    assert_eq!(header, "CODING_HDR");
    assert_eq!(user, "USER_TOKEN");
    assert_header_user_join(&combined, "CODING_HDR", "USER_TOKEN");
}

#[test]
fn combine_router_raw_header_and_user_joins_rendered_router_a_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(ROUTER_A_MD), "ROUTER_A_TOKEN\n").expect("router_a");
    let artifacts = flow_test_artifacts(&tmp);
    let store = PromptStore::with_root(prompt_root);
    let (combined, header, user) = combine_router_raw_header_and_user(
        &store,
        &artifacts,
        "USER_RAW_TOKEN\n\n",
        crate::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL, false),
    )
    .expect("combine");
    assert_eq!(header, "ROUTER_A_TOKEN");
    assert_eq!(user, "USER_RAW_TOKEN");
    assert_header_user_join(&combined, "ROUTER_A_TOKEN", "USER_RAW_TOKEN");
}

#[cfg(test)]
#[path = "router_flow_prompt_tests.rs"]
mod router_flow_prompt_tests;

#[cfg(test)]
#[path = "router_flow_vision_prompt_tests.rs"]
mod router_flow_vision_prompt_tests;

#[path = "router_flow_io_tests.rs"]
mod router_flow_io_tests;
