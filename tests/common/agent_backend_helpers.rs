//! Shared helpers for `AgentBackend` integration contract tests.

use malvin::acp::{AgentIoOptions, CoderPromptOptions};
use malvin::agent_backend::AgentBackend;
use malvin::mini_agent::{LlmBackend, MiniAgentClient, MiniLoopConfig, MiniRetryStrategy};
use malvin::openrouter_transport::CompletionResponse;

pub const NO_REAL_AGENT: &str = "MALVIN_TEST_NO_REAL_AGENT";

pub fn mini_done_wire() -> CompletionResponse {
    CompletionResponse {
        content: malvin::mini_agent::protocol::format_wire_turn("- done", "MINI_DONE"),
        usage: None,
        reasoning: None,
    }
}

pub const fn test_io() -> AgentIoOptions {
    AgentIoOptions {
        force: true,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

pub const fn tee_io() -> AgentIoOptions {
    AgentIoOptions {
        force: true,
        no_tee: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

pub fn mini_loop_config() -> MiniLoopConfig {
    MiniLoopConfig {
        model: "anthropic/claude-sonnet-4".into(),
        max_http_turns: 4,
        max_bash_execs: 128,
        max_http_retries: 1,
        max_transport_retries: 1,
        max_gate_retries: 1,
        max_shrink_passes: 0,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
        expects_investigation: false,
        allow_download: true,
        mini_constraints: "constraints".into(),
    }
}

pub fn mini_backend_with_llm(llm: LlmBackend, io: AgentIoOptions) -> AgentBackend {
    AgentBackend::Mini(
        MiniAgentClient::new(mini_loop_config(), io, llm).expect("mini client"),
    )
}

pub fn restore_acp_env(old_bin: Option<std::ffi::OsString>, old_no_real: Option<std::ffi::OsString>) {
    #[allow(unsafe_code)]
    unsafe {
        match old_bin {
            Some(v) => std::env::set_var("MALVIN_AGENT_ACP_BIN", v),
            None => std::env::remove_var("MALVIN_AGENT_ACP_BIN"),
        }
        match old_no_real {
            Some(v) => std::env::set_var(NO_REAL_AGENT, v),
            None => std::env::remove_var(NO_REAL_AGENT),
        }
    }
}

pub async fn run_lifecycle(backend: &mut AgentBackend, cwd: &std::path::Path, log: &std::path::Path) {
    backend.begin_coder_session(cwd).await.expect("begin");
    assert!(backend.has_open_coder_session());
    backend
        .run_coder_prompt("hello", log, "coder", CoderPromptOptions::default())
        .await
        .expect("prompt");
}

pub async fn finish_lifecycle(backend: &mut AgentBackend, expect_marker: &str) {
    let last = backend
        .last_coder_prompt_agent_response()
        .expect("last");
    assert!(last.contains(expect_marker), "last={last}");
    backend.end_coder_session().await.expect("end");
    assert!(!backend.has_open_coder_session());
}
