//! Shared helpers for `agent_backend` unit tests.

use std::path::PathBuf;

use crate::agent_backend::mini::{
    LlmBackend, LoopDriverConfig, LoopDriverSession, MiniLoopConfig, MiniRetryStrategy,
    MiniTraceSink, MockScript, MockStep,
};
use crate::cli::SharedOpts;
use malvin_mini::CompletionResponse;

#[must_use]
pub fn mini_done_response() -> CompletionResponse {
    CompletionResponse {
        content: "MINI_DONE".into(),
        usage: None,
        reasoning: None,
    }
}

#[must_use]
pub fn completion(content: impl Into<String>) -> CompletionResponse {
    CompletionResponse {
        content: content.into(),
        usage: None,
        reasoning: None,
    }
}

#[must_use]
pub fn mini_test_trace() -> MiniTraceSink {
    MiniTraceSink::new(None, test_io())
}

#[must_use]
pub fn mock_llm(responses: Vec<MockStep>) -> LlmBackend {
    LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses,
        call_count: 0,
        on_response: None,
    }))
}

#[must_use]
pub fn test_io() -> crate::acp::AgentIoOptions {
    crate::acp::AgentIoOptions {
        force: false,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

#[must_use]
pub fn loop_driver_config(max_http_turns: u32, max_http_retries: u32) -> LoopDriverConfig {
    LoopDriverConfig {
        max_http_turns,
        max_bash_execs: 128,
        max_http_retries,
        max_shrink_passes: 0,
        mini_constraints: "constraints",
        expects_investigation: false,
    }
}

#[must_use]
pub fn loop_session(cwd: PathBuf) -> LoopDriverSession {
    LoopDriverSession {
        messages: vec![],
        cwd,
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
        llm_model_slug: String::new(),
    }
}

#[must_use]
pub fn mini_loop_config(max_http_turns: u32, max_http_retries: u32) -> MiniLoopConfig {
    MiniLoopConfig {
        model: "anthropic/claude-sonnet-4".into(),
        max_http_turns,
        max_bash_execs: 128,
        max_http_retries,
        max_gate_retries: max_http_retries,
        max_shrink_passes: 0,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
        expects_investigation: false,
    }
}

#[must_use]
pub fn shared_opts(mini: bool) -> SharedOpts {
    SharedOpts {
        model: "auto".into(),
        no_force: false,
        no_tenacious: false,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: 3,
        doc: false,
        name: None,
        mini,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
    }
}

#[allow(unsafe_code)]
pub fn install_openrouter_test_key() {
    unsafe {
        std::env::set_var("OPENROUTER_API_KEY", "sk-test");
    }
}
