//! Build [`super::backend::AgentBackend`] from CLI options.

use crate::cli::{
    agent_io_options, new_agent_client, AgentStdoutTeeFlags, SharedOpts, WorkflowCliOptions,
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
            max_gate_retries: gate_retries,
            max_shrink_passes: shrink_passes,
            retry_strategy: MiniRetryStrategy::CumulativeTranscript,
            expects_investigation: false,
        },
        io,
    )
}
