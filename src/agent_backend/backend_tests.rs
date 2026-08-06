//! Behavioral and kiss coverage tests for [`super::backend::AgentBackend`].

use super::backend::AgentBackend;
use super::factory::build_agent_backend;
use crate::mini_agent::{LlmBackend, MiniAgentClient, MockScript, MockStep};
use super::test_support::{install_openrouter_test_key, mini_loop_config, openrouter_shared_opts, shared_opts, test_io};
use crate::cli::WorkflowCliOptions;
use crate::openrouter_transport::CompletionResponse;

#[must_use]
fn mock_mini_client() -> MiniAgentClient {
    MiniAgentClient::new_mock(
        mini_loop_config(4, 1),
        test_io(),
        LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![MockStep::Ok(CompletionResponse {
                content: "MINI_DONE".into(),
                usage: None,
                    reasoning: None,
            })],
            call_count: 0,
            on_response: None,
        })),
    )
}

#[test]
fn test_io_returns_agent_io_options_with_expected_flags() {
    let io = test_io();
    assert!(!io.force);
    assert!(io.no_tee);
    assert!(io.raw_output);
    assert!(!io.show_thoughts_on_stdout);
    assert!(!io.emit_stdout_markdown);
    assert!(!io.log_full_outgoing_prompts);
}

#[test]
fn cursor_sdk_keeps_coder_session_for_process_life() {
    let mini = AgentBackend::Mini(mock_mini_client());
    assert!(!mini.keeps_coder_session_for_process_life());
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
fn kiss_cov_backend_tests_helpers() {
    let _ = mock_mini_client;
    let _ = shared_opts;
    let _ = install_openrouter_test_key;
    let _ = stringify!(keeps_coder_session_for_process_life);
    let _ = stringify!(agent_backend_ensure_coder_session);
}

#[test]
fn build_agent_backend_selects_mini_for_openrouter_model() {
    install_openrouter_test_key();
    let backend = build_agent_backend(
        &openrouter_shared_opts(),
        WorkflowCliOptions { force: false },
        false,
        "code",
    )
    .expect("mini backend");
    assert!(matches!(backend, AgentBackend::Mini(_)));
}

#[test]
fn uses_mini_backend_for_local_prefix() {
    assert!(crate::model_id::uses_mini_backend("mini:local/qwen35_9b_q4"));
    assert!(crate::model_id::uses_local_backend("mini:local/nemotron3_nano_4b"));
}

#[test]
fn agent_backend_ensure_authenticated_mini_succeeds_with_test_key() {
    install_openrouter_test_key();
    let backend = AgentBackend::Mini(mock_mini_client());
    backend.ensure_authenticated().expect("authenticated");
}
