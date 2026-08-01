//! Shared helpers for [`super::backend_kpop_tests`] and error-path tests.

use std::sync::Mutex;

use super::backend::AgentBackend;
use crate::mini_agent::{LlmBackend, MiniAgentClient, MockScript, MockStep};
use super::test_support::{mini_done_response, mini_loop_config, test_io};
use crate::openrouter_transport::CompletionResponse;

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

pub(crate) fn mini_done_backend_multiturn() -> AgentBackend {
    mock_backend(
        vec![
            MockStep::Ok(mini_done_response()),
            MockStep::Ok(mini_done_response()),
            MockStep::Ok(mini_done_response()),
            MockStep::Ok(mini_done_response()),
        ],
        1,
    )
}

pub(crate) fn smoke_multiturn_state(
    _work: &std::path::Path,
    exp_log: std::path::PathBuf,
) -> crate::kpop_progression::KpopMultiturnState<'static> {
    use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};

    crate::kpop_progression::KpopMultiturnState::new(
        KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log,
        10,
    )
    .expect("state")
}

#[test]
fn smoke_multiturn_state_builds_state() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp = tmp.path().join("exp.md");
    std::fs::write(&exp, "# exp\n").expect("write");
    let mut state = smoke_multiturn_state(tmp.path(), exp);
    assert!(
        state.next_prompt().expect("first prompt").is_some(),
        "freshly constructed state should offer its first prompt"
    );
}

#[test]
fn smoke_mini_done_backend_multiturn_constructs() {
    let _backend = mini_done_backend_multiturn();
}
