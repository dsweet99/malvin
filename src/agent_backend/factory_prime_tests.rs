
use super::*;
use crate::agent_backend::test_support::shared_opts;
use crate::agent_backend::BridgeKind;
use crate::cli::WorkflowCliOptions;

#[test]
fn build_agent_backend_selects_prime_sdk() {
    let mut shared = shared_opts(false);
    shared.model = crate::model_id::parse_model_id("prime:openai/gpt-5.5").expect("model");
    let backend = build_agent_backend(
        &shared,
        WorkflowCliOptions { force: false },
        false,
        "code",
    )
    .expect("prime sdk");
    assert!(matches!(backend.kind, BridgeKind::Prime));
}

#[test]
fn prime_selected_even_when_legacy_acp_env_set() {
    // MALVIN_AGENT_ACP_BIN is ignored; product path is Cursor/Prime SDK only.
    crate::acp::with_env("MALVIN_AGENT_ACP_BIN", Some("/bin/true"), || {
        let mut shared = shared_opts(false);
        shared.model = crate::model_id::parse_model_id("prime:openai/gpt-5.5").expect("model");
        let backend = build_agent_backend(
            &shared,
            WorkflowCliOptions { force: false },
            false,
            "code",
        )
        .expect("prime");
        assert!(matches!(backend.kind, BridgeKind::Prime));
    });
}
