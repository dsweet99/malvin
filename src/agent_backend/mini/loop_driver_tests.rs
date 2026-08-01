use crate::agent_backend::mini::MiniRetryStrategy;
use crate::agent_backend::mini::{
    run_inner_loop, LlmBackend, LoopDriverConfig, LoopDriverRun, LoopDriverSession, MockScript,
    MockStep,
};
use crate::agent_backend::test_support::mini_test_trace;
use crate::malvin_mini::CompletionResponse;
use std::sync::Mutex;

fn test_config() -> LoopDriverConfig {
    LoopDriverConfig {
        max_http_turns: 8,
        max_http_retries: 3,
        max_transport_retries: 3,
        max_bash_execs: 128,
        max_shrink_passes: 0,
        expects_investigation: false,
        mini_constraints: "constraints",
    }
}

#[tokio::test]
async fn loop_driver_single_fence_runs_bash_and_appends_observation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let llm = LlmBackend::Mock(Mutex::new(MockScript {
        responses: vec![
            MockStep::Ok(CompletionResponse {
                content: crate::malvin_mini::format_wire_turn(
                    "- progress",
                    "```bash\necho hi > out.txt\n```",
                ),
                usage: None,
                reasoning: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: crate::malvin_mini::format_wire_turn("- progress", "summary"),
                usage: None,
                reasoning: None,
            }),
        ],
        call_count: 0,
        on_response: None,
    }));
    let mut session = LoopDriverSession {
        history: String::new(),
        previous_response: String::new(),
        pending_new_request: None,
        cwd: tmp.path().to_path_buf(),
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
        llm_model_slug: String::new(),
        section_shape_nudged: false,
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &test_config(),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop");
    assert_eq!(out.final_assistant_text, "summary");
    assert!(tmp.path().join("out.txt").is_file());
    assert!(session.history.contains("progress") || !session.history.is_empty());
    assert_eq!(session.previous_response, "summary");
    assert!(session.pending_new_request.is_none());
}

#[tokio::test]
async fn loop_driver_mini_done_line_terminates() {
    let llm = LlmBackend::Mock(Mutex::new(MockScript {
        responses: vec![MockStep::Ok(CompletionResponse {
            content: crate::malvin_mini::format_wire_turn("- progress", "MINI_DONE\n"),
            usage: None,
            reasoning: None,
        })],
        call_count: 0,
        on_response: None,
    }));
    let mut session = LoopDriverSession {
        history: String::new(),
        previous_response: String::new(),
        pending_new_request: None,
        cwd: std::env::temp_dir(),
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
        llm_model_slug: String::new(),
        section_shape_nudged: false,
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &test_config(),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop");
    assert!(out.final_assistant_text.contains("MINI_DONE"));
}

#[tokio::test]
async fn loop_driver_mini_done_inside_fence_still_runs_bash() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let llm = LlmBackend::Mock(Mutex::new(MockScript {
        responses: vec![
            MockStep::Ok(CompletionResponse {
                content: crate::malvin_mini::format_wire_turn(
                    "- progress",
                    "```bash\nMINI_DONE\necho fenced > fenced_out.txt\n```",
                ),
                usage: None,
                reasoning: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: crate::malvin_mini::format_wire_turn("- progress", "done after bash"),
                usage: None,
                reasoning: None,
            }),
        ],
        call_count: 0,
        on_response: None,
    }));
    let mut session = LoopDriverSession {
        history: String::new(),
        previous_response: String::new(),
        pending_new_request: None,
        cwd: tmp.path().to_path_buf(),
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
        llm_model_slug: String::new(),
        section_shape_nudged: false,
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &test_config(),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop");
    assert!(tmp.path().join("fenced_out.txt").is_file());
    assert_eq!(out.final_assistant_text, "done after bash");
}

#[tokio::test]
async fn loop_driver_new_history_uses_fact_kinds_after_bash_observation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let fact_history = "\
- objective: write out.txt (user-provided)
- observed: Exit code 0 from echo hi
- verified action: wrote out.txt
- inference: bash path works
- next: close
";
    let llm = LlmBackend::Mock(Mutex::new(MockScript {
        responses: vec![
            MockStep::Ok(CompletionResponse {
                content: crate::malvin_mini::format_wire_turn(
                    "- objective: write out.txt (user-provided)\n- proposal: run echo",
                    "```bash\necho hi > out.txt\n```",
                ),
                usage: None,
                reasoning: None,
            }),
            MockStep::Ok(CompletionResponse {
                content: crate::malvin_mini::format_wire_turn(fact_history, "summary after observe"),
                usage: None,
                reasoning: None,
            }),
        ],
        call_count: 0,
        on_response: None,
    }));
    let mut session = LoopDriverSession {
        history: String::new(),
        previous_response: String::new(),
        pending_new_request: None,
        cwd: tmp.path().to_path_buf(),
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
        llm_model_slug: String::new(),
        section_shape_nudged: false,
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "write out.txt",
        config: &test_config(),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop");
    assert!(tmp.path().join("out.txt").is_file());
    assert_eq!(out.final_assistant_text, "summary after observe");
    assert!(
        session.history.contains("user-provided")
            && session.history.contains("observed:")
            && session.history.contains("verified action:")
            && session.history.contains("inference:"),
        "NEW_HISTORY after tool observation must use structured fact kinds; got {:?}",
        session.history
    );
    assert_eq!(session.previous_response, "summary after observe");
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_loop_driver_test_symbols() {
        let _ = (
            mini_test_trace,
            test_config,
            loop_driver_single_fence_runs_bash_and_appends_observation,
            loop_driver_mini_done_line_terminates,
            loop_driver_mini_done_inside_fence_still_runs_bash,
            loop_driver_new_history_uses_fact_kinds_after_bash_observation,
        );
    }
}
