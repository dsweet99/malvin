//! Build [`super::backend::AgentBackend`] from CLI options.

use crate::cli::{agent_io_options, build_agent, AgentStdoutTeeFlags, SharedOpts, WorkflowCliOptions};

use super::backend::AgentBackend;
use super::mini::{MiniAgentClient, MiniLoopConfig};

const ALL_AGENT_COMMANDS: &[&str] = &[
    "do", "inspire", "plan", "code", "tidy", "delight", "explain", "revise", "init", "kpop",
];

/// # Errors
///
/// Returns an error when `--mini` is set but the command is not wired, or mini client init fails.
pub fn build_agent_backend(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    emit_stdout_markdown: bool,
    command: &str,
) -> Result<AgentBackend, String> {
    if shared.mini {
        mini_rollout_guard(command)?;
        Ok(AgentBackend::Mini(new_mini_client(
            shared,
            workflow,
            emit_stdout_markdown,
        )?))
    } else {
        Ok(AgentBackend::Acp(build_agent(
            shared,
            workflow,
            emit_stdout_markdown,
        )))
    }
}

fn new_mini_client(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    emit_stdout_markdown: bool,
) -> Result<MiniAgentClient, String> {
    let io = agent_io_options(
        shared,
        workflow,
        AgentStdoutTeeFlags {
            emit_stdout_markdown,
            raw_output: false,
            show_thoughts_on_stdout: true,
        },
    );
    MiniAgentClient::new(
        MiniLoopConfig {
            model: shared.model.clone(),
            max_bash_turns: shared.mini_max_bash_turns,
            max_http_retries: shared.max_acp_retries,
        },
        io,
    )
}

fn mini_rollout_guard(command: &str) -> Result<(), String> {
    if ALL_AGENT_COMMANDS.contains(&command) {
        Ok(())
    } else {
        Err(format!(
            "error: --mini is not yet supported for `malvin {command}` (implementation phase 4/4)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::WorkflowCliOptions;
    use crate::agent_backend::test_support::shared_opts;

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
    fn mini_rollout_guard_rejects_unwired_command() {
        let err = mini_rollout_guard("models").expect_err("models");
        assert!(err.contains("not yet supported"));
    }

    #[test]
    fn mini_rollout_guard_allows_wired_command_per_phase() {
        for cmd in ALL_AGENT_COMMANDS {
            mini_rollout_guard(cmd).expect(cmd);
        }
    }

    #[test]
    fn workspace_cargo_toml_lists_malvin_mini_member() {
        let text = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
        assert!(text.contains("malvin-mini"));
        assert!(text.contains("[workspace]"));
    }
}
