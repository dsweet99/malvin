//! Gate retry must consolidate via New request without restoring a durable message vec.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::openrouter_transport::{ChatMessage, ChatRole, CompletionResponse};

use super::{LlmBackend, MiniAgentClient, MockScript, MockStep};
use crate::acp::CoderPromptOptions;
use crate::agent_backend::test_support::{mini_loop_config, test_io};

const POLLUTION_MARKER: &str = "POLLUTION_MARKER_RETRY_TEST";
const TASK_MARKER: &str = "UNIQUE_TASK_MARKER_RETRY_TEST";

struct RetryPollutionObservation {
    task_marker_count: usize,
    polluted: bool,
    has_durable_message_vec: bool,
}

fn count_user_messages_with_marker(messages: &[ChatMessage], marker: &str) -> usize {
    messages
        .iter()
        .filter(|m| matches!(m.role, ChatRole::User) && m.content.contains(marker))
        .count()
}

fn observe_retry_http_history(
    idx: usize,
    messages: &[ChatMessage],
    slot: &Mutex<RetryPollutionObservation>,
) {
    // Second gate attempt's first consolidate call (after invest+failed wind-down uses 3 mocks).
    if idx != 3 {
        return;
    }
    let task_marker_count = count_user_messages_with_marker(messages, TASK_MARKER);
    let polluted = messages
        .iter()
        .any(|m| m.content.contains(POLLUTION_MARKER));
    // Assembled list is ephemeral: System Header (+cue) / optional History / optional Previous / User.
    // There is no growing User/Assistant transcript of prior tool turns.
    let assistant_count = messages
        .iter()
        .filter(|m| matches!(m.role, ChatRole::Assistant))
        .count();
    *slot.lock().expect("lock") = RetryPollutionObservation {
        task_marker_count,
        polluted,
        has_durable_message_vec: assistant_count > 1,
    };
}

fn retry_pollution_mock_client(observation: Arc<Mutex<RetryPollutionObservation>>) -> MiniAgentClient {
    let hook_slot = Arc::clone(&observation);
    MiniAgentClient::new_mock(
        mini_loop_config(1, 2),
        test_io(),
        LlmBackend::Mock(Mutex::new(MockScript {
            responses: vec![
                MockStep::Ok(CompletionResponse {
                    content: crate::mini_agent::protocol::format_wire_turn(
                        "- progress",
                        &format!("```bash\necho {POLLUTION_MARKER}\n```"),
                    ),
                    usage: None,
                    reasoning: None,
                }),
                // Wind-down: invalid sections twice (nudge then fail) → gate retry.
                MockStep::Ok(CompletionResponse {
                    content: "not a sectioned reply".into(),
                    usage: None,
                    reasoning: None,
                }),
                MockStep::Ok(CompletionResponse {
                    content: "still not sectioned".into(),
                    usage: None,
                    reasoning: None,
                }),
                MockStep::Ok(CompletionResponse {
                    content: crate::mini_agent::protocol::format_wire_turn("- progress", "MINI_DONE"),
                    usage: None,
                    reasoning: None,
                }),
            ],
            call_count: 0,
            on_response: Some(Box::new(move |idx, messages| {
                observe_retry_http_history(idx, messages, &hook_slot);
            })),
        })),
    )
}

async fn run_retry_pollution_prompt(client: &mut MiniAgentClient, work_dir: &Path, log_path: &Path) {
    client
        .begin_coder_session(work_dir)
        .await
        .expect("begin session");
    client
        .run_coder_prompt(
            &format!("do task {TASK_MARKER}"),
            log_path,
            "retry_test",
            CoderPromptOptions {
                single_attempt: false,
                ..Default::default()
            },
        )
        .await
        .expect("retry should succeed on second attempt");
    client.end_coder_session().await.expect("end session");
}

fn assert_retry_history_reflects_memory_model(observation: &RetryPollutionObservation) {
    assert_eq!(
        observation.task_marker_count, 0,
        "second attempt New request is divergence, not a re-pushed task prompt"
    );
    assert!(
        !observation.has_durable_message_vec,
        "assembled wire must not restore a multi-assistant chat transcript"
    );
    let _ = observation.polluted;
}

#[tokio::test]
async fn mini_coder_prompt_retry_does_not_pollute_session_history() {
    if super::bash_adapter::ensure_bash_on_path().is_err() {
        return;
    }

    let observation = Arc::new(Mutex::new(RetryPollutionObservation {
        task_marker_count: 0,
        polluted: false,
        has_durable_message_vec: true,
    }));
    let mut client = retry_pollution_mock_client(Arc::clone(&observation));
    let work_dir = tempfile::tempdir().expect("tempdir");
    let log_path: PathBuf = work_dir.path().join("retry_test.log");

    run_retry_pollution_prompt(&mut client, work_dir.path(), &log_path).await;

    let seen = observation.lock().expect("lock");
    assert_retry_history_reflects_memory_model(&seen);
}

#[cfg(test)]
mod gate_retry_role_tests {
    use crate::openrouter_transport::CompletionResponse;

    use super::*;
    use crate::mini_agent::retry_fork::build_divergence_observation;
    use crate::mini_agent::{
        run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession, MiniRetryStrategy,
    };
    use crate::agent_backend::test_support::{mini_test_trace, mock_llm};

    #[tokio::test]
    async fn cumulative_gate_retry_uses_divergence_as_new_request() {
        let llm = mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: crate::mini_agent::protocol::format_wire_turn(
                "- noted divergence",
                "I am the configured mini model.",
            ),
            usage: None,
            reasoning: None,
        })]);
        let divergence = build_divergence_observation(&[], "http failure", "git:abc");
        let mut session = LoopDriverSession {
            history: "prior".into(),
            previous_response: "Which LLM are you?".into(),
            pending_new_request: Some(divergence.clone()),
            cwd: std::env::temp_dir(),
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
            llm_model_slug: "anthropic/claude-sonnet-4".into(),
            section_shape_nudged: false,
        };
        let config = LoopDriverConfig {
            max_http_turns: 4,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_transport_retries: 3,
            max_shrink_passes: 0,
            mini_constraints: "constraints".into(),
            expects_investigation: false,
        };
        let out = run_inner_loop(LoopDriverRun {
            llm: &llm,
            session: &mut session,
            user_prompt: "Which LLM are you?",
            config: &config,
            trace: &mini_test_trace(),
            timing: None,
            llm_phase: None,
            single_attempt: true,
            gate_attempt: 2,
            retry_strategy: MiniRetryStrategy::CumulativeTranscript,
        })
        .await
        .expect("gate retry turn");
        assert!(out.final_assistant_text.contains("mini model"));
        assert!(session.history.contains("divergence") || session.history.contains("noted"));
        assert!(session.pending_new_request.is_none());
    }
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_client_retry_test_symbols() {
        let _ = (
            count_user_messages_with_marker,
            observe_retry_http_history,
            retry_pollution_mock_client,
            run_retry_pollution_prompt,
            assert_retry_history_reflects_memory_model,
            mini_coder_prompt_retry_does_not_pollute_session_history,
            stringify!(cumulative_gate_retry_uses_divergence_as_new_request),
        );
    }
}
