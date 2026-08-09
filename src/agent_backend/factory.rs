//! Build [`super::backend::AgentBackend`] from CLI options.

use crate::cli::{
    agent_io_options, default_workflow_stdout_tee_flags, new_agent_client, AgentStdoutTeeFlags,
    SharedOpts, WorkflowCliOptions,
};

use super::backend::AgentBackend;

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
    if crate::model_id::uses_prime_backend(&shared.model) {
        Ok(AgentBackend::PrimeSdk(new_prime_sdk_client(
            shared,
            agent_io_options(shared, workflow, tee),
        )))
    } else if cursor_acp_test_mock_override() {
        // Integration tests still install ACP JSON-RPC mocks via MALVIN_AGENT_ACP_BIN.
        // ACP override must not steal `prime:` (handled above).
        Ok(AgentBackend::Acp(new_agent_client(
            shared,
            agent_io_options(shared, workflow, tee),
        )))
    } else {
        Ok(AgentBackend::CursorSdk(new_cursor_sdk_client(
            shared,
            agent_io_options(shared, workflow, tee),
        )))
    }
}

fn cursor_acp_test_mock_override() -> bool {
    std::env::var_os("MALVIN_AGENT_ACP_BIN").is_some_and(|v| !v.is_empty())
}

fn new_cursor_sdk_client(
    shared: &SharedOpts,
    io: crate::acp::AgentIoOptions,
) -> crate::cursor_sdk::CursorSdkClient {
    crate::cursor_sdk::CursorSdkClient::with_max_retries(
        shared.model.clone(),
        io,
        shared.max_acp_retries,
    )
}

fn new_prime_sdk_client(
    shared: &SharedOpts,
    io: crate::acp::AgentIoOptions,
) -> crate::prime_sdk::PrimeSdkClient {
    let mut client = crate::prime_sdk::PrimeSdkClient::with_max_retries(
        shared.model.clone(),
        io,
        shared.max_acp_retries,
    );
    client.allow_download = !shared.no_download;
    client
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
        match backend {
            AgentBackend::CursorSdk(mut c) => {
                assert_eq!(
                    c.model, shared.model,
                    "CursorSdkClient must keep prefixed model id for COST rate lookup"
                );
                assert!(
                    c.model.contains(':'),
                    "expected prefixed model id, got {}",
                    c.model
                );
                // Attaching with the client model must resolve `[agent.cursor.auto]` rates.
                let timing = c.attach_run_timing_for_session();
                let rates = timing
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .token_cost_rates;
                let expected = crate::malvin_config_file::load_malvin_config(std::path::Path::new("."))
                    .token_cost_rates_for("cursor:auto");
                assert_eq!(rates, expected);
            }
            AgentBackend::Acp(_) | AgentBackend::PrimeSdk(_) => {
                panic!("expected CursorSdk backend")
            }
        }
    }

    #[test]
    fn agent_component_boundaries_are_in_tree_modules() {
        let text = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
        assert!(
            !text.contains("malvin-mini ="),
            "malvin must not path-depend on a separate malvin-mini crate"
        );
        assert!(
            std::path::Path::new("src/llm_transport/mod.rs").is_file(),
            "LlmTransport interface must live under src/llm_transport"
        );
        assert!(
            std::path::Path::new("src/local_llm/mod.rs").is_file(),
            "Local LLM transport must live under src/local_llm"
        );
        assert!(
            std::path::Path::new("src/agent/mod.rs").is_file(),
            "Agent interface must live under src/agent"
        );
        assert!(
            !std::path::Path::new("src/mini_agent/mod.rs").is_file(),
            "malvin-mini agent must be removed"
        );
        assert!(
            !std::path::Path::new("src/openrouter_transport/mod.rs").is_file(),
            "OpenRouter transport must be removed with malvin-mini"
        );
    }
}
