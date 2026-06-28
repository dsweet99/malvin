//! Cross-channel observability contract tests (narrative vs audit).
//!
//! Mini-only coverage for now; ACP-backed runs share the same channel split via
//! `PromptTraceWriter` (see [`malvin::observability`]).
mod common;

use std::time::Duration;

use common::mini_test_helpers::{mock_llm, trace_with_run_dir};
use common::observability_parity::{
    assert_audit_contains, assert_stdout_lacks_substring, assert_stdout_tool_vocab,
    trace_contains_substring,
};
use malvin::agent_backend::mini::{
    record_http_exchange, run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession,
    MiniHttpExchangeRecord, MiniTerminalReason, MiniTraceSink, MockStep,
};
use malvin::observability::audit_only_session_update_fields;
use malvin_mini::CompletionResponse;

#[test]
fn contract_plain_lines_bash_fence_audit_only_on_stdout() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let mut sink = trace_with_run_dir(&tmp, false);
    sink.plain_lines = true;
    sink.record_assistant_audit("```bash\ncat plan_dco.md\n```");
    assert_stdout_lacks_substring(&log_path, "plan_dco.md");
    assert_audit_contains(&tmp.path().join("trace.jsonl"), "agent_message_chunk");
    assert_audit_contains(&tmp.path().join("trace.jsonl"), "plan_dco.md");
    malvin::output::set_stdout_log_path(None);
}

#[test]
fn contract_audit_only_keys_never_in_stdout_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let sink = trace_with_run_dir(&tmp, true);
    record_http_exchange(
        &sink,
        MiniHttpExchangeRecord {
            attempt: 1,
            status: Some(502),
            body: Some("upstream"),
            error: Some("bad gateway".into()),
        },
    );
    let trace_path = tmp.path().join("trace.jsonl");
    assert_audit_contains(&trace_path, "miniHttpExchange");
    for field in audit_only_session_update_fields() {
        assert_stdout_lacks_substring(&log_path, field);
    }
    malvin::output::set_stdout_log_path(None);
}

#[tokio::test]
async fn contract_mini_terminal_in_trace_not_stdout() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let trace = trace_with_run_dir(&tmp, false);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
        llm_model_slug: String::new(),
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: "done without fence".into(),
            usage: None,
            reasoning: None,
        })]),
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_http_turns: 4,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_transport_retries: 3,
            max_shrink_passes: 0,
            mini_constraints: "c",
            expects_investigation: false,
        },
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: malvin::agent_backend::mini::MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop");
    assert_eq!(out.terminal.reason, MiniTerminalReason::FencelessComplete);
    let trace_path = tmp.path().join("trace.jsonl");
    trace_contains_substring(&trace_path, "miniTerminal");
    assert_stdout_lacks_substring(&log_path, "miniTerminal");
    assert_stdout_lacks_substring(&log_path, "fenceless_complete");
    malvin::output::set_stdout_log_path(None);
}

#[test]
fn contract_tagged_run_dual_emits_bash_to_both_channels() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let sink = MiniTraceSink::new(
        Some(tmp.path().to_path_buf()),
        malvin::acp::AgentIoOptions {
            force: false,
            no_tee: false,
            raw_output: false,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
    );
    sink.mini_bash_exec("echo hi", 0, Duration::from_millis(2), None);
    assert_stdout_tool_vocab(&log_path, &["Run"]);
    assert_audit_contains(&tmp.path().join("trace.jsonl"), "tool_call");
    malvin::output::set_stdout_log_path(None);
}

#[test]
fn contract_record_assistant_audit_leaves_stdout_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let sink = trace_with_run_dir(&tmp, false);
    sink.record_assistant_audit("secret audit-only body");
    assert_stdout_lacks_substring(&log_path, "secret audit-only body");
    assert_audit_contains(&tmp.path().join("trace.jsonl"), "secret audit-only body");
    malvin::output::set_stdout_log_path(None);
}
