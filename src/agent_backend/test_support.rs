use crate::acp::AgentIoOptions;
use crate::model_id::parse_model_id;

#[must_use]
pub fn test_io() -> AgentIoOptions {
    AgentIoOptions {
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

#[must_use]
pub fn sample_cursor_model() -> crate::model_id::ParsedModel {
    parse_model_id("cursor:auto").expect("model")
}
