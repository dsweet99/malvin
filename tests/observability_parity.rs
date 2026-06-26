//! Integration tests for `--mini` observability parity with ACP.

mod common;

use common::observability_parity::{
    assert_acp_trace_schema, assert_prompts_contains, assert_stdout_tool_vocab,
    stdout_m_before_t_on_multiturn, trace_contains_substring,
};
use malvin::agent_backend::mini::{
    run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession, LlmBackend, MiniTraceSink,
    MockScript, MockStep,
};
use malvin_mini::CompletionResponse;

const fn mini_io(no_tee: bool) -> malvin::acp::AgentIoOptions {
    malvin::acp::AgentIoOptions {
        force: false,
        no_tee,
        raw_output: !no_tee,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: true,
    }
}

fn trace_with_run_dir(tmp: &tempfile::TempDir, no_tee: bool) -> MiniTraceSink {
    MiniTraceSink::new(Some(tmp.path().to_path_buf()), mini_io(no_tee))
}

#[allow(clippy::missing_const_for_fn)]
fn mock_llm(steps: Vec<MockStep>) -> LlmBackend {
    LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses: steps,
        call_count: 0,
    }))
}

#[tokio::test]
async fn observability_parity_tool_log_includes_fence_comment() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let trace = trace_with_run_dir(&tmp, false);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
    };
    let target = tmp.path().join("seen.txt");
    std::fs::write(&target, "x").expect("write");

    run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![
            MockStep::Ok(CompletionResponse {
                content: format!(
                    "Inspect target file contents\n```bash\ncat {}\n```",
                    target.display()
                ),
                usage: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: "done".into(),
                usage: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: "done".into(),
                usage: None,
            }),
        ]),
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_bash_turns: 4,
            max_http_retries: 1,
            mini_constraints: "c",
        },
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    .expect("loop");

    let stdout = std::fs::read_to_string(log_path).expect("stdout");
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
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: "MINI_DONE\n".into(),
            usage: None,
        })]),
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_bash_turns: 4,
            max_http_retries: 1,
            mini_constraints: "MINI_CONSTRAINTS_MARKER",
        },
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    .expect("loop");

    assert!(!out.final_assistant_text.is_empty());

    let trace_path = tmp.path().join("trace.jsonl");
    assert_acp_trace_schema(&trace_path);
    trace_contains_substring(&trace_path, "agent_message_chunk");
}

#[tokio::test]
async fn observability_parity_nudge_in_prompts_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_with_run_dir(&tmp, true);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    let result = run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![
            MockStep::Ok(CompletionResponse {
                content: "no fence".into(),
                usage: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: "still no".into(),
                usage: None,
            }),
        ]),
        session: &mut session,
        user_prompt: "user bit",
        config: &LoopDriverConfig {
            max_bash_turns: 2,
            max_http_retries: 1,
            mini_constraints: "MINI_CONSTRAINTS_MARKER",
        },
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await;
    assert!(result.is_err(), "fenceless loop must exhaust turns");

    let prompts_path = tmp.path().join("prompts.log");
    assert_prompts_contains(
        &prompts_path,
        "your last response had no ```bash``` block",
    );
}

#[tokio::test]
async fn observability_parity_stdout_read_vocab_and_ordering() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let trace = trace_with_run_dir(&tmp, false);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
    };
    let target = tmp.path().join("target.txt");
    std::fs::write(&target, "content").expect("write");

    run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![
            MockStep::Ok(CompletionResponse {
                content: format!("```bash\ncat {}\n```", target.display()),
                usage: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: "done".into(),
                usage: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: "done".into(),
                usage: None,
            }),
        ]),
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_bash_turns: 4,
            max_http_retries: 1,
            mini_constraints: "c",
        },
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    .expect("loop");

    assert_stdout_tool_vocab(&log_path, &["Read"]);
    stdout_m_before_t_on_multiturn(&log_path);
    malvin::output::set_stdout_log_path(None);
}

#[tokio::test]
async fn observability_parity_no_tee_stdout_empty_trace_populated() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let trace = trace_with_run_dir(&tmp, true);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: "MINI_DONE".into(),
            usage: None,
        })]),
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_bash_turns: 4,
            max_http_retries: 1,
            mini_constraints: "c",
        },
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    .expect("loop");

    let stdout_text = std::fs::read_to_string(&log_path).unwrap_or_default();
    assert!(stdout_text.is_empty(), "no_tee must leave stdout.log empty");
    assert_acp_trace_schema(&tmp.path().join("trace.jsonl"));
    malvin::output::set_stdout_log_path(None);
}
