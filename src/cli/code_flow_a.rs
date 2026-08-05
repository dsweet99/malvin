use super::SharedOpts;

#[derive(Debug, Clone, Copy)]
pub struct WorkflowCliOptions {
    pub force: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct AgentStdoutTeeFlags {
    pub emit_stdout_markdown: bool,
    pub raw_output: bool,
    pub show_thoughts_on_stdout: bool,
}

/// Tee flags for the default workflow and for `malvin --verbose --do` (must stay identical).
#[must_use]
pub const fn default_workflow_stdout_tee_flags(emit_stdout_markdown: bool) -> AgentStdoutTeeFlags {
    AgentStdoutTeeFlags {
        emit_stdout_markdown,
        raw_output: false,
        show_thoughts_on_stdout: true,
    }
}

pub fn prepare_kpop_prompt_store(
    _workflow: WorkflowCliOptions,
    require_mbc2: bool,
) -> Result<crate::prompts::PromptStore, String> {
    use crate::prompts::{PromptError, PromptStore};
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation { require_mbc2 })
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn agent_io_options(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> crate::acp::AgentIoOptions {
    crate::acp::AgentIoOptions {
        force: workflow.force,
        no_tee: crate::output::stdout_suppressed(),
        raw_output: tee.raw_output,
        show_thoughts_on_stdout: tee.show_thoughts_on_stdout,
        emit_stdout_markdown: tee.emit_stdout_markdown,
        log_full_outgoing_prompts: shared.verbose,
    }
}

pub fn format_workspace_gate_failure(command: &str, detail: &str) -> String {
    format!(
        "ERR: Workspace checks did not pass; the next step did not run.\n\
Run `malvin tidy`, then retry `{command}`.\n\
\n\
{detail}"
    )
}

pub fn new_agent_client(
    shared: &SharedOpts,
    io: crate::acp::AgentIoOptions,
) -> crate::acp::AgentClient {
    crate::acp::AgentClient::with_max_acp_retries(
        crate::model_id::provider_slug(&shared.model),
        io,
        shared.max_acp_retries,
    )
}

pub fn build_agent(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    emit_stdout_markdown: bool,
) -> crate::acp::AgentClient {
    new_agent_client(
        shared,
        agent_io_options(
            shared,
            workflow,
            default_workflow_stdout_tee_flags(emit_stdout_markdown),
        ),
    )
}
