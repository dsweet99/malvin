//! Agent ACP behavioral smokes for `kiss check` coverage (split from `coverage_kiss` for file size limits).

use crate::acp::{AgentClient, AgentIoOptions, has_api_key};

#[test]
fn smoke_agent_client_new_has_no_open_coder_session() {
    let io = AgentIoOptions {
        force: false,
        sandbox: false,
        no_tee: false,
        raw_output: false,
        show_thoughts_on_stdout: true,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    };
    let client = AgentClient::new("smoke-model".to_string(), io);
    assert!(!client.has_open_coder_session());
}

#[test]
fn smoke_has_api_key_reads_env_without_panic() {
    let _: bool = has_api_key();
}
