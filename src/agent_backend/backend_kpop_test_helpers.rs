//! Shared helpers for [`super::backend_kpop_tests`] and error-path tests.

use std::sync::Mutex;

use super::backend::AgentBackend;
use super::mini::{LlmBackend, MiniAgentClient, MockScript, MockStep};
use super::test_support::{mini_done_response, mini_loop_config, test_io};
use malvin_mini::CompletionResponse;

pub(crate) fn mock_backend(responses: Vec<MockStep>, max_http_retries: u32) -> AgentBackend {
    AgentBackend::Mini(MiniAgentClient::new_mock(
        mini_loop_config(4, max_http_retries),
        test_io(),
        LlmBackend::Mock(Mutex::new(MockScript {
            responses,
            call_count: 0,
            on_response: None,
        })),
    ))
}

pub(crate) fn mock_backend_bash_turn_exhaustion() -> AgentBackend {
    AgentBackend::Mini(MiniAgentClient::new_mock(
        mini_loop_config(1, 1),
        test_io(),
        LlmBackend::Mock(Mutex::new(MockScript {
            responses: vec![MockStep::Ok(CompletionResponse {
                content: "```bash\necho hi\n```".into(),
                usage: None,
                reasoning: None,
            })],
            call_count: 0,
            on_response: None,
        })),
    ))
}

pub(crate) fn empty_backups() -> crate::artifacts::SessionDotfileBackups {
    crate::orchestrator::orchestrator_test_support::empty_dotfile_backups()
}

pub(crate) fn mini_done_backend() -> AgentBackend {
    mock_backend(vec![MockStep::Ok(mini_done_response())], 1)
}
