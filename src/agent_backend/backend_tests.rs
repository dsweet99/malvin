//! Behavioral and kiss coverage tests for [`super::backend::AgentBackend`].

use super::backend::AgentBackend;
use super::factory::build_agent_backend;
use super::mini::{LlmBackend, MiniAgentClient, MiniLoopConfig, MockScript, MockStep};
use super::test_support::{install_openrouter_test_key, shared_opts, test_io};
use crate::cli::WorkflowCliOptions;
use malvin_mini::CompletionResponse;

#[must_use]
fn mock_mini_client() -> MiniAgentClient {
    MiniAgentClient::new_mock(
        MiniLoopConfig {
            model: "anthropic/claude-sonnet-4".into(),
            max_bash_turns: 4,
            max_http_retries: 1,
        },
        test_io(),
        LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![MockStep::Ok(CompletionResponse {
                content: "MINI_DONE".into(),
                usage: None,
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
fn kiss_cov_backend_tests_helpers() {
    let _ = mock_mini_client;
    let _ = shared_opts;
    let _ = install_openrouter_test_key;
}

#[test]
fn build_agent_backend_selects_mini_when_mini_true() {
    install_openrouter_test_key();
    let backend = build_agent_backend(
        &shared_opts(true),
        WorkflowCliOptions { force: false },
        false,
        "code",
    )
    .expect("mini backend");
    assert!(matches!(backend, AgentBackend::Mini(_)));
}

#[test]
fn agent_backend_ensure_authenticated_mini_succeeds_with_test_key() {
    install_openrouter_test_key();
    let backend = AgentBackend::Mini(mock_mini_client());
    backend.ensure_authenticated().expect("authenticated");
}
