//! Shared helpers for `agent_backend` unit tests.

use crate::acp::AgentIoOptions;
use crate::cli::SharedOpts;
use crate::agent_backend::mini::MiniTraceSink;
use malvin_mini::CompletionResponse;

#[must_use]
pub fn mini_done_response() -> CompletionResponse {
    CompletionResponse {
        content: "MINI_DONE".into(),
        usage: None,
    }
}

#[must_use]
pub fn mini_test_trace() -> MiniTraceSink {
    MiniTraceSink {
        run_dir: None,
        io: test_io(),
    }
}

#[must_use]
pub fn test_io() -> AgentIoOptions {
    AgentIoOptions {
        force: false,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
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
    }
}

#[allow(unsafe_code)]
pub fn install_openrouter_test_key() {
    unsafe {
        std::env::set_var("OPENROUTER_API_KEY", "sk-test");
    }
}
