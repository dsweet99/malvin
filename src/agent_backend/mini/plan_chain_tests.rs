//! `PromptChain` topology: plan stages share one mini message history per session.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use malvin_mini::{ChatMessage, CompletionResponse};

use super::{LlmBackend, MiniAgentClient, MiniLoopConfig, MockScript, MockStep};
use crate::acp::CoderPromptOptions;
use crate::agent_backend::test_support::test_io;

const MARKER: &str = "PLAN_CHAIN_MARKER";

fn bash_then_final_responses() -> Vec<MockStep> {
    vec![
        MockStep::Ok(CompletionResponse {
            content: format!("```bash\necho {MARKER}\n```"),
            usage: None,
        }),
        MockStep::Ok(CompletionResponse {
            content: "summary for prompt 1a".into(),
            usage: None,
        }),
        // Fence-less reply after bash + no-fence nudge (see `classify_turn`).
        MockStep::Ok(CompletionResponse {
            content: "final summary for prompt 1a after nudge".into(),
            usage: None,
        }),
    ]
}

fn plan_chain_all_responses() -> Vec<MockStep> {
    let mut responses = bash_then_final_responses();
    // Prompt 3: fence-less first reply triggers one no-fence nudge (same as prompt 1a).
    responses.push(MockStep::Ok(CompletionResponse {
        content: "summary for prompt 3".into(),
        usage: None,
    }));
    responses.push(MockStep::Ok(CompletionResponse {
        content: "MINI_DONE\nfinal summary for prompt 3 after nudge".into(),
        usage: None,
    }));
    responses
}

fn record_marker_at_prompt3_idx3(seen: &Mutex<bool>, idx: usize, messages: &[ChatMessage]) {
    if idx != 3 {
        return;
    }
    let history = messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<String>();
    if history.contains(MARKER) {
        let mut g = seen.lock().expect("lock");
        *g = true;
    }
}

fn plan_chain_mock_client(seen_marker: Arc<Mutex<bool>>) -> MiniAgentClient {
    let hook_seen = Arc::clone(&seen_marker);
    MiniAgentClient::new_mock(
        MiniLoopConfig {
            model: "anthropic/claude-sonnet-4".into(),
            max_bash_turns: 8,
            max_http_retries: 1,
        },
        test_io(),
        LlmBackend::Mock(Mutex::new(MockScript {
            responses: plan_chain_all_responses(),
            call_count: 0,
            on_response: Some(Box::new(move |idx, messages| {
                record_marker_at_prompt3_idx3(&hook_seen, idx, messages);
            })),
        })),
    )
}

struct PlanChainWorkDirs {
    work_dir: tempfile::TempDir,
    log_1a: PathBuf,
    log_3: PathBuf,
}

fn plan_chain_work_dirs() -> PlanChainWorkDirs {
    let work_dir = tempfile::tempdir().expect("tempdir");
    let log_1a = work_dir.path().join("plan_1a.log");
    let log_3 = work_dir.path().join("plan_3.log");
    PlanChainWorkDirs {
        work_dir,
        log_1a,
        log_3,
    }
}

async fn run_plan_chain_test_prompt_1a(client: &mut MiniAgentClient, work_dir: &Path, log_1a: &Path) {
    client.begin_coder_session(work_dir).await.expect("begin");
    client
        .run_coder_prompt(
            "plan prompt 1a",
            log_1a,
            "plan_1a",
            CoderPromptOptions::default(),
        )
        .await
        .expect("prompt 1a");
}

async fn run_plan_chain_test_prompt_3(client: &mut MiniAgentClient, log_3: &Path) {
    client
        .run_coder_prompt(
            "plan prompt 3",
            log_3,
            "plan_3",
            CoderPromptOptions::default(),
        )
        .await
        .expect("prompt 3");
    client.end_coder_session().await.expect("end");
}

#[tokio::test]
async fn plan_prompt_chain_shared_history() {
    if super::bash_adapter::ensure_bash_on_path().is_err() {
        return;
    }

    let seen_marker = Arc::new(Mutex::new(false));
    let mut client = plan_chain_mock_client(Arc::clone(&seen_marker));
    let dirs = plan_chain_work_dirs();

    run_plan_chain_test_prompt_1a(&mut client, dirs.work_dir.path(), &dirs.log_1a).await;
    assert!(
        !*seen_marker.lock().expect("lock"),
        "marker should not appear in history until prompt 3's first HTTP round"
    );

    run_plan_chain_test_prompt_3(&mut client, &dirs.log_3).await;
    assert!(
        *seen_marker.lock().expect("lock"),
        "prompt 3 HTTP round should see bash observation from prompt 1a in shared history"
    );
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_plan_chain_test_symbols() {
        let _ = (
            bash_then_final_responses,
            plan_chain_all_responses,
            record_marker_at_prompt3_idx3,
            plan_chain_mock_client,
            plan_chain_work_dirs,
            run_plan_chain_test_prompt_1a,
            run_plan_chain_test_prompt_3,
            plan_prompt_chain_shared_history,
        );
    }
}
