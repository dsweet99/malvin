//! Shared helpers for mini integration tests.

pub const fn mini_io(no_tee: bool) -> malvin::acp::AgentIoOptions {
    malvin::acp::AgentIoOptions {
        force: false,
        no_tee,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: true,
    }
}

pub fn trace_with_run_dir(
    tmp: &tempfile::TempDir,
    no_tee: bool,
) -> malvin::mini_agent::MiniTraceSink {
    malvin::mini_agent::MiniTraceSink::new(Some(tmp.path().to_path_buf()), mini_io(no_tee))
}

#[allow(clippy::missing_const_for_fn)]
pub fn mock_llm(
    steps: Vec<malvin::mini_agent::MockStep>,
) -> malvin::mini_agent::LlmBackend {
    malvin::mini_agent::LlmBackend::Mock(std::sync::Mutex::new(
        malvin::mini_agent::MockScript {
            responses: steps,
            call_count: 0,
        },
    ))
}

pub fn parity_session(cwd: &std::path::Path) -> malvin::mini_agent::LoopDriverSession {
    malvin::mini_agent::LoopDriverSession {
        history: String::new(),
        previous_response: String::new(),
        pending_new_request: None,
        cwd: cwd.to_path_buf(),
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
        llm_model_slug: String::new(),
        section_shape_nudged: false,
    }
}

pub fn parity_loop_config(mini_constraints: &'static str) -> malvin::mini_agent::LoopDriverConfig {
    malvin::mini_agent::LoopDriverConfig {
        max_http_turns: 4,
        max_http_retries: 1,
        max_transport_retries: 3,
        max_bash_execs: 128,
        max_shrink_passes: 0,
        expects_investigation: false,
        mini_constraints: mini_constraints.to_string(),
    }
}

pub fn read_stdout_log(log_path: &std::path::Path) -> String {
    std::fs::read_to_string(log_path).expect("stdout")
}

pub async fn run_parity_bash_loop(
    tmp: &tempfile::TempDir,
    log_path: &std::path::Path,
    target: &std::path::Path,
    fence_comment: &str,
) {
    malvin::output::set_stdout_log_path(Some(log_path.to_path_buf()));
    let trace = trace_with_run_dir(tmp, false);
    let mut session = parity_session(tmp.path());
    let config = parity_loop_config("c");
    let llm = mock_llm(vec![
        malvin::mini_agent::MockStep::Ok(malvin::openrouter_transport::CompletionResponse {
            content: malvin::mini_agent::protocol::format_wire_turn(
                "- progress",
                &format!(
                    "{fence_comment}\n```bash\ncat {}\n```",
                    target.display()
                ),
            ),
            usage: None,
            reasoning: None,
        }),
        malvin::mini_agent::MockStep::Ok(malvin::openrouter_transport::CompletionResponse {
            content: malvin::mini_agent::protocol::format_wire_turn("- progress", "done"),
            usage: None,
            reasoning: None,
        }),
    ]);
    malvin::mini_agent::run_inner_loop(malvin::mini_agent::LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &config,
        trace: &trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: malvin::mini_agent::MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop");
}

pub struct ParityLoopInput<'a> {
    pub llm: &'a malvin::mini_agent::LlmBackend,
    pub session: &'a mut malvin::mini_agent::LoopDriverSession,
    pub user_prompt: &'a str,
    pub config: &'a malvin::mini_agent::LoopDriverConfig,
    pub trace: &'a malvin::mini_agent::MiniTraceSink,
}

#[allow(clippy::missing_const_for_fn, clippy::needless_pass_by_value)]
pub fn parity_loop_run(input: ParityLoopInput<'_>) -> malvin::mini_agent::LoopDriverRun<'_> {
    malvin::mini_agent::LoopDriverRun {
        llm: input.llm,
        session: input.session,
        user_prompt: input.user_prompt,
        config: input.config,
        trace: input.trace,
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: malvin::mini_agent::MiniRetryStrategy::CumulativeTranscript,
    }
}

pub async fn run_parity_mock_loop(
    input: ParityLoopInput<'_>,
) -> malvin::mini_agent::LoopDriverOutcome {
    malvin::mini_agent::run_inner_loop(parity_loop_run(input))
        .await
        .expect("parity loop")
}

pub fn parity_stdout_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    malvin::output::set_stdout_log_path(Some(log_path.clone()));
    (tmp, log_path)
}

pub fn clear_stdout_fixture() {
    malvin::output::set_stdout_log_path(None);
}
