
use super::*;
use crate::agent_backend::test_support::shared_opts;
use crate::cli::WorkflowCliOptions;

#[test]
fn build_agent_backend_selects_prime_sdk() {
    let mut shared = shared_opts(false);
    shared.model = "prime:openai/gpt-5.5".into();
    let backend = build_agent_backend(
        &shared,
        WorkflowCliOptions { force: false },
        false,
        "code",
    )
    .expect("prime sdk");
    assert!(matches!(backend, AgentBackend::PrimeSdk(_)));
}

#[test]
fn prime_not_stolen_by_acp_env() {
    crate::acp::with_env("MALVIN_AGENT_ACP_BIN", Some("/bin/true"), || {
        let mut shared = shared_opts(false);
        shared.model = "prime:openai/gpt-5.5".into();
        let backend = build_agent_backend(
            &shared,
            WorkflowCliOptions { force: false },
            false,
            "code",
        )
        .expect("prime");
        assert!(matches!(backend, AgentBackend::PrimeSdk(_)));
    });
}
