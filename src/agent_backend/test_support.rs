//! Shared helpers for `agent_backend` unit tests.

use crate::cli::SharedOpts;

#[must_use]
pub fn test_io() -> crate::acp::AgentIoOptions {
    crate::acp::AgentIoOptions {
        // Match cursor/prime mock clients: sessions require tool auto-run (`--force` default).
        force: true,
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
        no_force: false,
        no_tenacious: false,
        gates: false,
        quiet: false,
        verbose: false,
        max_acp_retries: 3,
        doc: false,
        name: None,
        no_download: false,
        git: false,
    }
}
