use super::SharedOpts;

#[derive(Debug, Clone, Copy)]
pub struct WorkflowCliOptions {
    pub force: bool,
    pub run_learn: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct AgentStdoutTeeFlags {
    pub emit_stdout_markdown: bool,
    pub raw_output: bool,
    pub show_thoughts_on_stdout: bool,
}

pub fn prepare_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<crate::prompts::PromptStore, String> {
    use crate::prompts::{PromptError, PromptStore};
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store.validate_required().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("summary.md")
        .map_err(|e: PromptError| e.0)?;
    if workflow.run_learn {
        store
            .validate_exists("learn.md")
            .map_err(|e: PromptError| e.0)?;
    }
    Ok(store)
}

pub fn prepare_bug_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<crate::prompts::PromptStore, String> {
    use crate::prompts::PromptError;
    let store = prepare_prompt_store(workflow)?;
    store
        .validate_exists("bug_regression_test.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("bug_fix.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn prepare_kpop_prompt_store(
    workflow: WorkflowCliOptions,
    require_mbc2: bool,
) -> Result<crate::prompts::PromptStore, String> {
    use crate::prompts::{PromptError, PromptStore};
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
            run_learn: workflow.run_learn,
            require_mbc2,
        })
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub const fn agent_io_options(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> crate::acp::AgentIoOptions {
    crate::acp::AgentIoOptions {
        force: workflow.force,
        no_tee: shared.no_tee,
        raw_output: tee.raw_output,
        show_thoughts_on_stdout: tee.show_thoughts_on_stdout,
        emit_stdout_markdown: tee.emit_stdout_markdown,
        log_full_outgoing_prompts: shared.verbose,
    }
}

pub fn format_pre_check_gate_failure(command: &str, detail: &str) -> String {
    format!(
        "ERR: Pre-checks failed; implementation did not start.\n\
Run `malvin tidy`, then retry `{command}`, or use `--skip-pre-checks` on `{command}`.\n\
\n\
{detail}"
    )
}

pub fn format_workspace_gate_failure(command: &str, detail: &str) -> String {
    format!(
        "ERR: Workspace checks did not pass; the next step did not run.\n\
Run `malvin tidy`, then retry `{command}`, or use `--skip-pre-checks` on `{command}`.\n\
\n\
{detail}"
    )
}

pub fn format_code_pre_check_failure(detail: &str) -> String {
    format_pre_check_gate_failure("malvin code", detail)
}

pub fn build_agent(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    emit_stdout_markdown: bool,
) -> crate::acp::AgentClient {
    crate::acp::AgentClient::new(
        shared.model.to_string(),
        agent_io_options(
            shared,
            workflow,
            AgentStdoutTeeFlags {
                emit_stdout_markdown,
                raw_output: false,
                show_thoughts_on_stdout: true,
            },
        ),
    )
}
