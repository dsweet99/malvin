//! Build [`super::backend::AgentBackend`] from CLI options.

use crate::cli::{
    agent_io_options, default_workflow_stdout_tee_flags, new_agent_client, AgentStdoutTeeFlags,
    SharedOpts, WorkflowCliOptions,
};

use super::backend::AgentBackend;
use super::mini::{MiniAgentClient, MiniLoopConfig, MiniRetryStrategy};

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
        default_workflow_stdout_tee_flags(emit_stdout_markdown),
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
    if crate::model_id::uses_mini_backend(&shared.model) {
        Ok(AgentBackend::Mini(new_mini_client(shared, workflow, tee)?))
    } else {
        Ok(AgentBackend::Acp(new_agent_client(
            shared,
            agent_io_options(shared, workflow, tee),
        )))
    }
}

#[allow(clippy::missing_const_for_fn)]
fn mini_http_turns(shared: &SharedOpts) -> u32 {
    // `--mini-max-bash-turns` is a deprecated alias for HTTP turns.
    if shared.mini_max_bash_turns == 32 {
        shared.mini_max_http_turns
    } else {
        shared.mini_max_bash_turns
    }
}

const fn mini_gate_retries(shared: &SharedOpts) -> u32 {
    if shared.mini_max_gate_retries > 0 {
        shared.mini_max_gate_retries
    } else {
        shared.max_acp_retries
    }
}

fn new_mini_client(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> Result<MiniAgentClient, String> {
    let io = agent_io_options(shared, workflow, tee);
    let tenacious = !shared.no_tenacious;
    let http_retries = if tenacious && shared.mini_max_http_retries == 0 {
        9999
    } else {
        shared.mini_max_http_retries
    };
    let gate_retries = if tenacious && shared.mini_max_gate_retries == 0 && shared.max_acp_retries <= 3 {
        9999
    } else {
        mini_gate_retries(shared)
    };
    let transport_retries = shared.mini_max_transport_retries.max(1);
    let shrink_passes = if tenacious && shared.mini_max_shrink_passes == 0 {
        3
    } else {
        shared.mini_max_shrink_passes
    };
    MiniAgentClient::new(
        MiniLoopConfig {
            model: shared.model.clone(),
            max_http_turns: mini_http_turns(shared),
            max_bash_execs: shared.mini_max_bash_execs,
            max_http_retries: http_retries,
            max_transport_retries: transport_retries,
            max_gate_retries: gate_retries,
            max_shrink_passes: shrink_passes,
            retry_strategy: MiniRetryStrategy::CumulativeTranscript,
            expects_investigation: false,
            allow_download: !shared.no_download,
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
    fn malvin_crate_embeds_malvin_mini_module_not_path_dep() {
        let text = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
        assert!(
            !text.contains("malvin-mini ="),
            "malvin must not path-depend on a separate malvin-mini crate"
        );
        assert!(
            std::path::Path::new("src/malvin_mini/mod.rs").is_file(),
            "malvin-mini sources must live under src/malvin_mini"
        );
    }
}
