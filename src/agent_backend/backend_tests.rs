//! Behavioral and kiss coverage tests for [`super::backend::AgentBackend`].

use super::backend::AgentBackend;
use super::factory::build_agent_backend;
use super::test_support::{shared_opts, test_io};
use crate::cli::WorkflowCliOptions;

#[test]
fn test_io_returns_agent_io_options_with_expected_flags() {
    let io = test_io();
    assert!(io.force);
    assert!(io.no_tee);
    assert!(io.raw_output);
    assert!(!io.show_thoughts_on_stdout);
    assert!(!io.emit_stdout_markdown);
    assert!(!io.log_full_outgoing_prompts);
}

#[test]
fn cursor_and_prime_keep_coder_session_for_process_life() {
    let acp = AgentBackend::Acp(crate::acp::AgentClient::new(
        "auto".into(),
        test_io(),
    ));
    assert!(!acp.keeps_coder_session_for_process_life());
    let sdk = AgentBackend::CursorSdk(crate::cursor_sdk::CursorSdkClient::new(
        "cursor:auto".into(),
        test_io(),
    ));
    assert!(sdk.keeps_coder_session_for_process_life());
    let prime = AgentBackend::PrimeSdk(crate::prime_sdk::PrimeSdkClient::new(
        "prime:openai/gpt-4o".into(),
        test_io(),
    ));
    assert!(prime.keeps_coder_session_for_process_life());
}

#[test]
fn build_agent_backend_selects_prime_for_prime_model() {
    let mut shared = shared_opts(false);
    shared.model = "prime:openai/gpt-4o".into();
    let backend = build_agent_backend(
        &shared,
        WorkflowCliOptions { force: false },
        false,
        "code",
    )
    .expect("prime backend");
    assert!(matches!(backend, AgentBackend::PrimeSdk(_)));
}

#[test]
fn uses_local_backend_for_prime_local_prefix() {
    assert!(crate::model_id::uses_local_backend("prime:local/qwen35_9b_q4"));
    assert!(!crate::model_id::uses_local_backend("prime:openai/gpt-4o"));
}
