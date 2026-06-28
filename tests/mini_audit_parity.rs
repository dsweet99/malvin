//! Audit parity: `miniTerminal`, shrink, and fork events in `trace.jsonl`.

mod common;

use common::mini_test_helpers::{mock_llm, trace_with_run_dir};
use common::observability_parity::{
    assert_acp_trace_schema, trace_contains_substring,
};
use malvin::agent_backend::mini::{
    run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession,
    MiniPhase, MiniTerminalReason, MockStep,
};
use malvin_mini::CompletionResponse;

#[tokio::test]
async fn mini_audit_fenceless_complete_emits_mini_terminal() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_with_run_dir(&tmp, true);
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

    assert_eq!(
        out.terminal.reason,
        MiniTerminalReason::FencelessComplete
    );
    let trace_path = tmp.path().join("trace.jsonl");
    assert_acp_trace_schema(&trace_path);
    trace_contains_substring(&trace_path, "miniTerminal");
    trace_contains_substring(&trace_path, "fenceless_complete");
}

#[tokio::test]
async fn mini_audit_mini_done_emits_mini_terminal() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_with_run_dir(&tmp, true);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
    llm_model_slug: String::new(),
    };
    run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: "MINI_DONE\n".into(),
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

    let trace_path = tmp.path().join("trace.jsonl");
    trace_contains_substring(&trace_path, "mini_done_outside_fence");
}

#[tokio::test]
async fn mini_audit_context_overflow_with_zero_shrink_passes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_with_run_dir(&tmp, true);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
    llm_model_slug: String::new(),
    };
    match run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![MockStep::ContextOverflow]),
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
    {
        Err(e) => assert!(e.0.contains("overflow")),
        Ok(_) => panic!("expected context overflow"),
    }
    let trace_path = tmp.path().join("trace.jsonl");
    trace_contains_substring(&trace_path, "context_overflow");
}

#[tokio::test]
async fn mini_audit_wind_down_after_bash_on_last_http_turn() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_with_run_dir(&tmp, true);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
    llm_model_slug: String::new(),
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![
            MockStep::Ok(CompletionResponse {
                content: "```bash\necho wind > wind.txt\n```".into(),
                usage: None,
                reasoning: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: "wind down summary".into(),
                usage: None,
                reasoning: None,
            }),
        ]),
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_http_turns: 1,
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
    .expect("wind down");
    assert_eq!(out.terminal.http_turn_count, 2);
    assert_eq!(out.terminal.phase_at_exit, MiniPhase::WindDown);
    assert_eq!(out.terminal.reason, MiniTerminalReason::FencelessComplete);
    assert!(tmp.path().join("wind.txt").is_file());
    trace_contains_substring(&tmp.path().join("trace.jsonl"), "wind_down");
}

#[tokio::test]
async fn mini_audit_no_tee_stdout_empty_trace_populated() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    let trace = trace_with_run_dir(&tmp, true);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
    llm_model_slug: String::new(),
    };
    run_inner_loop(LoopDriverRun {
        llm: &mock_llm(vec![MockStep::Ok(CompletionResponse {
            content: "MINI_DONE".into(),
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

    let stdout_text = std::fs::read_to_string(&log_path).unwrap_or_default();
    assert!(stdout_text.is_empty(), "no_tee must leave stdout.log empty");
    assert_acp_trace_schema(&tmp.path().join("trace.jsonl"));
    malvin::output::set_stdout_log_path(None);
}
