//! Build [`super::backend::AgentBackend`] from CLI options.

use crate::cli::{
    agent_io_options, default_workflow_stdout_tee_flags, AgentStdoutTeeFlags, SharedOpts,
    WorkflowCliOptions,
};
use crate::model_id::ModelBackend;

use super::backend::AgentBackend;
use super::sdk_client::{BridgeKind, SdkClient};

/// # Errors
///
/// Returns an error when backend construction fails.
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

/// Like [`build_agent_backend`] but accepts explicit stdout tee flags (for example `do` raw mode).
///
/// # Errors
///
/// Returns an error when backend construction fails.
pub fn build_agent_backend_with_tee(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> Result<AgentBackend, String> {
    let model = shared.model.clone();
    let io = agent_io_options(shared, workflow, tee);
    let kind = match model.backend {
        ModelBackend::Cursor => BridgeKind::Cursor,
        ModelBackend::Prime => BridgeKind::Prime,
    };
    let mut client = SdkClient::with_max_retries(model, kind, io, shared.max_acp_retries);
    if matches!(kind, BridgeKind::Prime) {
        client.allow_download = !shared.no_download;
    }
    Ok(crate::agent_backend::agent_backend_from_client(client))
}

#[cfg(test)]
#[path = "factory_prime_tests.rs"]
mod factory_prime_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_backend::test_support::shared_opts;
    use crate::cli::WorkflowCliOptions;

    #[test]
    fn build_agent_backend_selects_cursor_sdk() {
        let shared = shared_opts(false);
        let backend = build_agent_backend(
            &shared,
            WorkflowCliOptions { force: false },
            false,
            "code",
        )
        .expect("cursor sdk");
        assert!(matches!(backend.kind, BridgeKind::Cursor));
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
    fn build_agent_backend_selects_prime_when_prefixed() {
        let mut shared = shared_opts(false);
        shared.model = crate::model_id::parse_model_id("prime:openai/gpt-4o").expect("model");
        let backend = build_agent_backend(
            &shared,
            WorkflowCliOptions { force: false },
            false,
            "code",
        )
        .expect("prime sdk");
        assert!(matches!(backend.kind, BridgeKind::Prime));
    }
}
