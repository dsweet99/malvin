//! Mini vs ACP narrative stdout parity tests.

mod common;

use common::mini_test_helpers::{
    mock_llm, parity_loop_config, parity_session, read_stdout_log, run_parity_bash_loop,
    trace_with_run_dir,
};
use common::observability_parity::{
    assert_acp_trace_schema, assert_stdout_tool_vocab,
    stdout_m_before_t_on_multiturn, trace_contains_substring,
};
use malvin::agent_backend::mini::{
    run_inner_loop, LoopDriverRun, MiniRetryStrategy, MockStep,
};
use malvin_mini::CompletionResponse;

#[tokio::test]
async fn observability_parity_tool_log_includes_fence_comment() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    let target = tmp.path().join("seen.txt");
    std::fs::write(&target, "x").expect("write");
    run_parity_bash_loop(&tmp, &log_path, &target, "Inspect target file contents").await;

    let stdout = read_stdout_log(&log_path);
    assert!(
        stdout.contains("Inspect target file contents"),
        "tool log must carry fence comment; got {stdout:?}"
    );
    malvin::output::set_stdout_log_path(None);
}

#[tokio::test]
async fn observability_parity_trace_acp_schema_after_mock_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_with_run_dir(&tmp, true);
    let mut session = parity_session(tmp.path());
    let config = parity_loop_config("MINI_CONSTRAINTS_MARKER");
    let out = run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: "MINI_DONE\n".into(),
            usage: None,
            reasoning: None,
        })]),
        session: &mut session,
        user_prompt: "go",
        config: &config,
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop");

    assert!(!out.final_assistant_text.is_empty());

    let trace_path = tmp.path().join("trace.jsonl");
    assert_acp_trace_schema(&trace_path);
    trace_contains_substring(&trace_path, "agent_message_chunk");
}

#[tokio::test]
async fn observability_parity_fenceless_completes_in_one_turn() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_with_run_dir(&tmp, true);
    let mut session = parity_session(std::env::temp_dir().as_path());
    let config = parity_loop_config("MINI_CONSTRAINTS_MARKER");
    let out = run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: "informational answer".into(),
            usage: None,
            reasoning: None,
        })]),
        session: &mut session,
        user_prompt: "Hello. What kind of LLM are you?",
        config: &config,
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("fenceless informational prompt completes in one turn");
    assert_eq!(out.final_assistant_text, "informational answer");

    let prompts_path = tmp.path().join("prompts.log");
    let prompts = std::fs::read_to_string(&prompts_path).unwrap_or_default();
    assert!(
        !prompts.contains("your last response had no ```bash``` block"),
        "no-fence nudge must not appear in prompts.log"
    );
}

#[tokio::test]
async fn observability_parity_stdout_read_vocab_and_ordering() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    let target = tmp.path().join("target.txt");
    std::fs::write(&target, "content").expect("write");
    run_parity_bash_loop(&tmp, &log_path, &target, "").await;

    assert_stdout_tool_vocab(&log_path, &["Read"]);
    stdout_m_before_t_on_multiturn(&log_path);
    malvin::output::set_stdout_log_path(None);
}
