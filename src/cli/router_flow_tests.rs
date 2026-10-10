use crate::cli::router_flow::router_flow_prompt::{
    RouterAPromptInput, RouterHeaderPromptInput, build_router_a_prompt, build_router_header_prompt,
    prepare_router_prompt_store,
};
use malvin::config::DEFAULT_CLI_MODEL;
use malvin::flow_prompt_join_test_helpers::flow_test_artifacts;
use malvin::prompts::{HEADER_MD, ROUTER_A_MD, ROUTER_B_CREATIVE_LEAD_MD, ROUTER_B_MD};

#[test]
fn prepare_router_prompt_store_loads_default_templates() {
    let store = prepare_router_prompt_store().expect("store");
    assert!(store.validate_exists(HEADER_MD).is_ok());
    assert!(store.validate_exists(ROUTER_A_MD).is_ok());
    assert!(store.validate_exists(ROUTER_B_MD).is_ok());
    assert!(store.validate_exists(ROUTER_B_CREATIVE_LEAD_MD).is_ok());
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
    })
    .expect("header");
    assert!(!body.is_empty());
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_header_prompt_embeds_workspace_agents_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    std::fs::write(tmp.path().join("AGENTS.md"), "Record the map scale.\n").expect("agents");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_header_prompt(RouterHeaderPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
    })
    .expect("header");
    assert!(
        body.contains("## Workspace `AGENTS.md`") && body.contains("Record the map scale."),
        "router header must embed workspace AGENTS.md via agents_insert: {body}"
    );
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
    })
    .expect("router_a");
    assert!(body.contains("plan.md"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_a_prompt_does_not_inline_request_text() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    std::fs::write(&artifacts.plan_path, "Please fix A5-marker.").expect("write plan");
    let store = prepare_router_prompt_store().expect("store");
    let render = |model: &str| {
        build_router_a_prompt(RouterAPromptInput {
            store: &store,
            artifacts: &artifacts,
            model,
            gates: false,
            gates_just_ran: false,
        })
        .expect("router_a")
    };
    for model in ["pi:local/ollama/malvin-llama32:latest", DEFAULT_CLI_MODEL] {
        let body = render(model);
        assert!(!body.contains("A5-marker"), "{model}: {body}");
        assert!(!body.contains("request_inline"), "{model}: {body}");
    }
}

#[cfg(test)]
#[path = "router_flow_prompt_tests.rs"]
mod router_flow_prompt_tests;

#[cfg(test)]
#[path = "router_flow_vision_prompt_tests.rs"]
mod router_flow_vision_prompt_tests;

#[path = "router_flow_io_tests.rs"]
mod router_flow_io_tests;
