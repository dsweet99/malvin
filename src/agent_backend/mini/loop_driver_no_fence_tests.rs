use crate::agent_backend::mini::MiniRetryStrategy;
use crate::agent_backend::mini::MiniTraceSink;
use super::{
    run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession, MockStep,
};
use crate::agent_backend::test_support::{mini_test_trace, mock_llm};
use malvin_mini::CompletionResponse;

#[tokio::test]
async fn loop_driver_fenceless_completes_in_one_turn() {
    let llm = mock_llm(vec![MockStep::Ok(CompletionResponse {
        content: "informational answer".into(),
        usage: None,
                    reasoning: None,
    })]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
    llm_model_slug: String::new(),
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "Hello. What kind of LLM are you?",
        config: &LoopDriverConfig {
            max_http_turns: 4,
            max_http_retries: 3,
            max_transport_retries: 3,
            max_bash_execs: 128,
            max_shrink_passes: 0,
            expects_investigation: false,
            mini_constraints: "constraints",
        },
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
    gate_attempt: 1,
    retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("single-turn fenceless must not re-call LLM");
    assert_eq!(out.final_assistant_text, "informational answer");
    assert!(
        !session
            .messages
            .iter()
            .any(|m| m.content.contains("your last response had no ```bash``` block")),
        "no-fence nudge must not be injected"
    );
}

#[tokio::test]
async fn loop_driver_fenceless_no_nudge_in_prompts_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = MiniTraceSink::new(Some(tmp.path().to_path_buf()), crate::acp::AgentIoOptions {
        force: false,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    });
    let llm = mock_llm(vec![MockStep::Ok(CompletionResponse {
        content: "done in one turn".into(),
        usage: None,
                    reasoning: None,
    })]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
    llm_model_slug: String::new(),
    };
    run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_http_turns: 4,
            max_http_retries: 3,
            max_transport_retries: 3,
            max_bash_execs: 128,
            max_shrink_passes: 0,
            expects_investigation: false,
            mini_constraints: "constraints",
        },
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
    gate_attempt: 1,
    retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop ok");
    let prompts = std::fs::read_to_string(tmp.path().join("prompts.log")).unwrap_or_default();
    assert!(
        !prompts.contains("your last response had no ```bash``` block"),
        "prompts.log must not contain no-fence nudge"
    );
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_no_fence_test_symbols() {
        let _ = (
            loop_driver_fenceless_completes_in_one_turn,
            loop_driver_fenceless_no_nudge_in_prompts_log,
        );
    }
}
