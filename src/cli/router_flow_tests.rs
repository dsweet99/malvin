use crate::router_flow::router_flow_prompt::{
    RouterAPromptInput, RouterHeaderPromptInput, RouterKpopCommonPromptInput,
    build_router_a_prompt, build_router_header_prompt, build_router_kpop_common_prompt,
    prepare_router_prompt_store,
};
use malvin::config::DEFAULT_CLI_MODEL;
use malvin::flow_prompt_join_test_helpers::flow_test_artifacts;
use malvin::prompts::{HEADER_MD, ROUTER_A_MD, ROUTER_B_CREATIVE_MD, ROUTER_B_MD};

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
        max_hypotheses: 5,
        no_kpop: false,
        gate_iteration: 1,
    })
    .expect("header");
    assert!(body.contains("Know thyself") || body.contains("Context Prep") || !body.is_empty());
    assert!(!body.contains("{{"));
    assert!(
        body.contains("KPop") || body.contains("Karl Popper"),
        "header must include kpop_insert when no_kpop is false"
    );
}

#[test]
fn build_router_header_prompt_embeds_workspace_agents_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    std::fs::write(tmp.path().join("AGENTS.md"), "Prefer ripwire for maps.\n").expect("agents");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_header_prompt(RouterHeaderPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        max_hypotheses: 5,
        no_kpop: true,
        gate_iteration: 1,
    })
    .expect("header");
    assert!(
        body.contains("## Workspace `AGENTS.md`") && body.contains("Prefer ripwire for maps."),
        "router header must embed workspace AGENTS.md via agents_insert: {body}"
    );
}

#[test]
fn build_router_header_prompt_no_kpop_leaves_kpop_insert_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_header_prompt(RouterHeaderPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        max_hypotheses: 5,
        no_kpop: true,
        gate_iteration: 1,
    })
    .expect("header no_kpop");
    assert!(!body.contains("{{"));
    assert!(
        !body.contains("Karl Popper") && !body.to_ascii_lowercase().contains("falsifiable"),
        "no_kpop must resolve kpop_insert to empty: {body}"
    );
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
        max_hypotheses: 7,
        no_kpop: false,
        gate_iteration: 1,
    })
    .expect("kpop common");
    assert!(body.contains("max_hypotheses = `7`"));
    assert!(body.contains("exp_log_"));
    assert!(body.contains("_g1.md") || body.contains("_g1"));
    assert!(!body.contains("{{"));
    let body2 = build_router_kpop_common_prompt(RouterKpopCommonPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        max_hypotheses: 7,
        no_kpop: false,
        gate_iteration: 2,
    })
    .expect("kpop common g2");
    assert!(body2.contains("_g2.md") || body2.contains("_g2"));
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
        gates: false,
        gates_just_ran: false,
        no_kpop: false,
    })
    .expect("router_a");
    assert!(body.contains("__MALVIN_DONE__"));
    assert!(!body.contains("{{"));
}

#[cfg(test)]
#[path = "router_flow_prompt_tests.rs"]
mod router_flow_prompt_tests;

#[cfg(test)]
#[path = "router_flow_vision_prompt_tests.rs"]
mod router_flow_vision_prompt_tests;

#[path = "router_flow_io_tests.rs"]
mod router_flow_io_tests;
