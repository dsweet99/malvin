use crate::acp::AgentIoOptions;
use crate::model_id::ParsedModel;

use super::sdk_client::SdkClient;

#[derive(Debug, Clone, Copy)]
pub struct AgentStdoutTeeFlags {
    pub emit_stdout_markdown: bool,
    pub raw_output: bool,
    pub show_thoughts_on_stdout: bool,
}

#[must_use]
pub const fn default_workflow_stdout_tee_flags(emit_stdout_markdown: bool) -> AgentStdoutTeeFlags {
    AgentStdoutTeeFlags {
        emit_stdout_markdown,
        raw_output: false,
        show_thoughts_on_stdout: true,
    }
}

#[must_use]
pub fn agent_io_options(
    log_full_outgoing_prompts: bool,
    tee: AgentStdoutTeeFlags,
) -> AgentIoOptions {
    AgentIoOptions {
        no_tee: crate::output::stdout_suppressed(),
        raw_output: tee.raw_output,
        show_thoughts_on_stdout: tee.show_thoughts_on_stdout,
        emit_stdout_markdown: tee.emit_stdout_markdown,
        log_full_outgoing_prompts,
    }
}

pub fn build_agent_backend(
    model: ParsedModel,
    max_acp_retries: u32,
    emit_stdout_markdown: bool,
) -> Result<SdkClient, String> {
    build_agent_backend_with_tee(
        model,
        max_acp_retries,
        default_workflow_stdout_tee_flags(emit_stdout_markdown),
        false,
    )
}

pub fn build_agent_backend_with_tee(
    model: ParsedModel,
    max_acp_retries: u32,
    tee: AgentStdoutTeeFlags,
    log_full_outgoing_prompts: bool,
) -> Result<SdkClient, String> {
    let io = agent_io_options(log_full_outgoing_prompts, tee);
    Ok(SdkClient::with_max_retries(model, io, max_acp_retries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_id::{ModelBackend, parse_model_id};

    fn sample_model(id: &str) -> ParsedModel {
        parse_model_id(id).expect("model")
    }

    #[test]
    fn build_agent_backend_selects_cursor_sdk() {
        let model = sample_model("cursor:auto");
        let backend = build_agent_backend(model.clone(), 3, false).expect("cursor sdk");
        assert!(matches!(backend.model.backend, ModelBackend::Cursor));
        assert_eq!(
            backend.model.canonical(),
            model.canonical(),
            "SdkClient must keep prefixed model id for COST rate lookup"
        );
        assert!(
            backend.model.canonical().contains(':'),
            "expected prefixed model id, got {}",
            backend.model.canonical()
        );
        let mut client = backend;
        let timing = client.attach_run_timing_for_session();
        let rates = timing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .token_cost_rates;
        let expected = crate::malvin_config_file::load_malvin_config(std::path::Path::new("."))
            .token_cost_rates_for("cursor:auto");
        assert_eq!(rates, expected);
    }

    #[test]
    fn build_agent_backend_selects_pi_when_prefixed() {
        let model = sample_model("rpi:openai/gpt-4o");
        let backend = build_agent_backend(model, 3, false).expect("pi sdk");
        assert!(matches!(backend.model.backend, ModelBackend::Pi));
        assert_eq!(backend.model.canonical(), "rpi:openai/gpt-4o");
    }

    #[test]
    fn build_agent_backend_selects_npm_pi_when_prefixed() {
        let model = sample_model("pi:openai/gpt-4o");
        let backend = build_agent_backend(model, 3, false).expect("npm pi");
        assert!(matches!(backend.model.backend, ModelBackend::NpmPi));
        assert_eq!(backend.model.canonical(), "pi:openai/gpt-4o");
    }

    #[test]
    fn build_agent_backend_selects_codex_when_prefixed() {
        let model = sample_model("codex:gpt-5.6");
        let backend = build_agent_backend(model, 3, false).expect("codex sdk");
        assert!(matches!(backend.model.backend, ModelBackend::Codex));
        assert_eq!(backend.model.canonical(), "codex:gpt-5.6");
    }
}
