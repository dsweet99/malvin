use crate::cli::{
    AgentStdoutTeeFlags, SharedOpts, WorkflowCliOptions, agent_io_options,
    default_workflow_stdout_tee_flags,
};

use super::backend::AgentBackend;
use super::sdk_client::SdkClient;

pub fn build_agent_backend(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    emit_stdout_markdown: bool,
    _command: &str,
) -> Result<AgentBackend, String> {
    build_agent_backend_with_tee(
        shared,
        workflow,
        default_workflow_stdout_tee_flags(emit_stdout_markdown),
    )
}

pub fn build_agent_backend_with_tee(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> Result<AgentBackend, String> {
    let model = shared.model.clone();
    let io = agent_io_options(shared, workflow, tee);
    let client = SdkClient::with_max_retries(model, io, shared.max_acp_retries);
    Ok(crate::agent_backend::agent_backend_from_client(client))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_backend::test_support::shared_opts;
    use crate::cli::WorkflowCliOptions;
    use crate::model_id::ModelBackend;

    #[test]
    fn build_agent_backend_selects_cursor_sdk() {
        let shared = shared_opts(false);
        let backend =
            build_agent_backend(&shared, WorkflowCliOptions { force: false }, false, "code")
                .expect("cursor sdk");
        assert!(matches!(backend.model.backend, ModelBackend::Cursor));
        assert_eq!(
            backend.model.canonical(),
            shared.model.canonical(),
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
        let mut shared = shared_opts(false);
        shared.model = crate::model_id::parse_model_id("pi:openai/gpt-4o").expect("model");
        let backend =
            build_agent_backend(&shared, WorkflowCliOptions { force: false }, false, "code")
                .expect("pi sdk");
        assert!(matches!(backend.model.backend, ModelBackend::Pi));
        assert_eq!(backend.model.canonical(), "pi:openai/gpt-4o");
    }

    #[test]
    fn build_agent_backend_selects_codex_when_prefixed() {
        let mut shared = shared_opts(false);
        shared.model = crate::model_id::parse_model_id("codex:gpt-5.6").expect("model");
        let backend =
            build_agent_backend(&shared, WorkflowCliOptions { force: false }, false, "code")
                .expect("codex sdk");
        assert!(matches!(backend.model.backend, ModelBackend::Codex));
        assert_eq!(backend.model.canonical(), "codex:gpt-5.6");
    }
}
