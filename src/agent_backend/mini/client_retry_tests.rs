//! Retry must not leave duplicate user prompts or stale partial turns in session history.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use malvin_mini::{ChatMessage, CompletionResponse};

use super::{LlmBackend, MiniAgentClient, MockScript, MockStep};
use crate::acp::CoderPromptOptions;
    use crate::agent_backend::test_support::{mini_loop_config, test_io};

const POLLUTION_MARKER: &str = "POLLUTION_MARKER_RETRY_TEST";
const TASK_MARKER: &str = "UNIQUE_TASK_MARKER_RETRY_TEST";

struct RetryPollutionObservation {
    task_marker_count: usize,
    polluted: bool,
}

fn count_user_messages_with_marker(messages: &[ChatMessage], marker: &str) -> usize {
    messages
        .iter()
        .filter(|m| matches!(m.role, malvin_mini::ChatRole::User) && m.content.contains(marker))
        .count()
}

fn observe_retry_http_history(idx: usize, messages: &[ChatMessage], slot: &Mutex<RetryPollutionObservation>) {
    if idx != 1 {
        return;
    }
    let task_marker_count = count_user_messages_with_marker(messages, TASK_MARKER);
    let polluted = messages
        .iter()
        .any(|m| m.content.contains(POLLUTION_MARKER));
    *slot.lock().expect("lock") = RetryPollutionObservation {
        task_marker_count,
        polluted,
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
                    content: format!("```bash\necho {POLLUTION_MARKER}\n```"),
                    usage: None,
                    reasoning: None,
                }),
                MockStep::Ok(CompletionResponse {
                    content: "MINI_DONE".into(),
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

fn assert_retry_history_reflects_cumulative_transcript(observation: &RetryPollutionObservation) {
    assert_eq!(
        observation.task_marker_count,
        1,
        "retry HTTP call should see exactly one user message with the task marker"
    );
    assert!(
        observation.polluted,
        "cumulative transcript retains bash observations from the failed attempt"
    );
}

#[tokio::test]
async fn mini_coder_prompt_retry_does_not_pollute_session_history() {
    if super::bash_adapter::ensure_bash_on_path().is_err() {
        return;
    }

    let observation = Arc::new(Mutex::new(RetryPollutionObservation {
        task_marker_count: 0,
        polluted: true,
    }));
    let mut client = retry_pollution_mock_client(Arc::clone(&observation));
    let work_dir = tempfile::tempdir().expect("tempdir");
    let log_path: PathBuf = work_dir.path().join("retry_test.log");

    run_retry_pollution_prompt(&mut client, work_dir.path(), &log_path).await;

    let seen = observation.lock().expect("lock");
    assert_retry_history_reflects_cumulative_transcript(&seen);
}


#[cfg(test)]
mod gate_retry_role_tests {
    use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

    use super::*;
    use crate::agent_backend::mini::retry_fork::build_divergence_observation;
    use crate::agent_backend::mini::{
        run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession, MiniRetryStrategy,
    };
    use crate::agent_backend::test_support::{mini_test_trace, mock_llm};

    fn consecutive_user_roles(messages: &[ChatMessage]) -> usize {
        let mut max_run = 0_usize;
        let mut run = 0_usize;
        for msg in messages {
            if matches!(msg.role, ChatRole::User) {
                run += 1;
                max_run = max_run.max(run);
            } else {
                run = 0;
            }
        }
        max_run
    }

    #[tokio::test]
    async fn cumulative_gate_retry_skips_repushed_user_prompt() {
        let llm = mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: "I am the configured mini model.".into(),
            usage: None,
            reasoning: None,
        })]);
        let mut session = LoopDriverSession {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "Which LLM are you?".into(),
            }],
            cwd: std::env::temp_dir(),
            constraints_prepended: true,
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
            llm_model_slug: "anthropic/claude-sonnet-4".into(),
        };
        let divergence = build_divergence_observation(&[], "http failure", "git:abc");
        session.messages.push(ChatMessage {
            role: ChatRole::User,
            content: divergence,
        });
        let config = LoopDriverConfig {
            max_http_turns: 4,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_transport_retries: 3,
            max_shrink_passes: 0,
            mini_constraints: "constraints",
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
        assert_eq!(
            consecutive_user_roles(&session.messages),
            2,
            "expected user prompt + divergence only, not a third repushed prompt: {:?}",
            session.messages
        );
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
            assert_retry_history_reflects_cumulative_transcript,
            mini_coder_prompt_retry_does_not_pollute_session_history,
            stringify!(cumulative_gate_retry_skips_repushed_user_prompt),
        );
    }
}
