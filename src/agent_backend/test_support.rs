use crate::cli::SharedOpts;

#[must_use]
pub fn test_io() -> crate::acp::AgentIoOptions {
    crate::acp::AgentIoOptions {
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

#[must_use]
pub fn shared_opts(_unused: bool) -> SharedOpts {
    SharedOpts {
        model: crate::model_id::parse_model_id("cursor:auto").expect("model"),
        verbose: false,
        max_acp_retries: 3,
        doc: false,
    }
}
