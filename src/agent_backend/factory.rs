//! Build [`super::backend::AgentBackend`] from CLI options.

use crate::cli::{
    agent_io_options, new_agent_client, AgentStdoutTeeFlags, SharedOpts, WorkflowCliOptions,
};

use super::backend::AgentBackend;
use super::mini::{MiniAgentClient, MiniLoopConfig};

/// # Errors
///
/// Returns an error when mini client init fails (for example missing `OPENROUTER_API_KEY` or `bash`).
pub fn build_agent_backend(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    emit_stdout_markdown: bool,
    _command: &str,
) -> Result<AgentBackend, String> {
    build_agent_backend_with_tee(
        shared,
        workflow,
        AgentStdoutTeeFlags {
            emit_stdout_markdown,
            raw_output: false,
            show_thoughts_on_stdout: true,
        },
    )
}

/// Like [`build_agent_backend`] but accepts explicit stdout tee flags (for example `do` raw mode).
///
/// # Errors
///
/// Returns an error when mini client init fails.
pub fn build_agent_backend_with_tee(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> Result<AgentBackend, String> {
    if shared.mini {
        Ok(AgentBackend::Mini(new_mini_client(shared, workflow, tee)?))
    } else {
        Ok(AgentBackend::Acp(new_agent_client(
            shared,
            agent_io_options(shared, workflow, tee),
        )))
    }
}

fn new_mini_client(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> Result<MiniAgentClient, String> {
    let io = agent_io_options(shared, workflow, tee);
    MiniAgentClient::new(
        MiniLoopConfig {
            model: shared.model.clone(),
            max_bash_turns: shared.mini_max_bash_turns,
            max_http_retries: shared.max_acp_retries,
        },
        io,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_backend::test_support::{install_openrouter_test_key, shared_opts};
    use crate::cli::WorkflowCliOptions;

    #[test]
    fn build_agent_backend_selects_acp_when_mini_false() {
        let backend = build_agent_backend(
            &shared_opts(false),
            WorkflowCliOptions { force: false },
            false,
            "code",
        )
        .expect("acp");
        assert!(matches!(backend, AgentBackend::Acp(_)));
    }

    #[test]
    fn build_agent_backend_with_tee_selects_mini_when_flag_set() {
        install_openrouter_test_key();
        let backend = build_agent_backend_with_tee(
            &shared_opts(true),
            WorkflowCliOptions { force: false },
            AgentStdoutTeeFlags {
                emit_stdout_markdown: false,
                raw_output: true,
                show_thoughts_on_stdout: false,
            },
        )
        .expect("mini");
        assert!(matches!(backend, AgentBackend::Mini(_)));
    }

    #[test]
    fn workspace_cargo_toml_lists_malvin_mini_member() {
        let text = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
        assert!(text.contains("malvin-mini"));
        assert!(text.contains("[workspace]"));
    }
}
