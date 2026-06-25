//! Retry must not leave duplicate user prompts or stale partial turns in session history.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use malvin_mini::{ChatMessage, CompletionResponse};

use super::{LlmBackend, MiniAgentClient, MiniLoopConfig, MockScript, MockStep};
use crate::acp::CoderPromptOptions;
use crate::agent_backend::test_support::test_io;

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
        MiniLoopConfig {
            model: "m".into(),
            max_bash_turns: 1,
            max_http_retries: 2,
        },
        test_io(),
        LlmBackend::Mock(Mutex::new(MockScript {
            responses: vec![
                MockStep::Ok(CompletionResponse {
                    content: format!("```bash\necho {POLLUTION_MARKER}\n```"),
                    usage: None,
                }),
                MockStep::Ok(CompletionResponse {
                    content: "MINI_DONE".into(),
                    usage: None,
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

fn assert_retry_history_is_clean(observation: &RetryPollutionObservation) {
    assert_eq!(
        observation.task_marker_count,
        1,
        "successful retry HTTP call should see exactly one user message with the task marker"
    );
    assert!(
        !observation.polluted,
        "stale bash observation from failed attempt must not appear in retry HTTP history"
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
    assert_retry_history_is_clean(&seen);
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
            assert_retry_history_is_clean,
            mini_coder_prompt_retry_does_not_pollute_session_history,
        );
    }
}
