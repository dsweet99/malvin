use malvin::acp::AgentClient;
use malvin::artifacts::{
    RunArtifacts, backup_workspace_grounding_if_present, create_run_artifacts_from_text,
    resolve_user_request,
};
use malvin::orchestrator::{Orchestrator, WorkflowConfig, WorkflowError};
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{PromptError, PromptStore};

use super::{CodeArgs, SharedOpts};
use super::repo_checks::{run_repo_workspace_gates, RepoGateOutput};
use super::{run_emit, timing_merge};

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

pub fn prepare_prompt_store(workflow: WorkflowCliOptions) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store.validate_required().map_err(|e: PromptError| e.0)?;
    if workflow.run_learn {
        store
            .validate_exists("learn.md")
            .map_err(|e: PromptError| e.0)?;
    }
    Ok(store)
}

pub fn prepare_kpop_prompt_store(
    workflow: WorkflowCliOptions,
    require_mbc2: bool,
) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_kpop_prompts(malvin::prompts::KpopPromptValidation {
            run_learn: workflow.run_learn,
            require_mbc2,
        })
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn prepare_code_run(
    code: &CodeArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(PromptStore, AgentClient, RunArtifacts), String> {
    let store = prepare_prompt_store(workflow)?;
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let client = build_agent(shared, workflow, emit_stdout_markdown);
    let (text, work_dir) = resolve_user_request(&code.request)?;
    let artifacts = create_run_artifacts_from_text(&text, Some(work_dir.as_path()))
        .map_err(|e| e.to_string())?;
    Ok((store, client, artifacts))
}

pub const fn agent_io_options(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> malvin::acp::AgentIoOptions {
    malvin::acp::AgentIoOptions {
        force: workflow.force,
        no_tee: shared.no_tee,
        raw_output: tee.raw_output,
        show_thoughts_on_stdout: tee.show_thoughts_on_stdout,
        emit_stdout_markdown: tee.emit_stdout_markdown,
    }
}

pub fn build_agent(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    emit_stdout_markdown: bool,
) -> AgentClient {
    AgentClient::new(
        shared.model.clone(),
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

pub async fn run_code(
    code: super::CodeArgs,
    shared: &super::SharedOpts,
    workflow: super::WorkflowCliOptions,
) -> Result<(), String> {
    let (store, mut client, artifacts) = prepare_code_run(&code, shared, workflow)?;
    run_repo_workspace_gates(&artifacts.work_dir, RepoGateOutput::Tagged)?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    run_emit::emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), &code.request)?;
    let mut orch = Orchestrator {
        client: &mut client,
        prompts: &store,
        artifacts: &artifacts,
        config: WorkflowConfig {
            max_loops: code.max_loops,
            run_learn: workflow.run_learn,
            learn_min_elapsed_ms: 300_000,
            skip_check_plan: code.trust_the_plan,
        },
        progress_callback: Box::new(|msg: &str| {
            print_stdout_line(MALVIN_WHO, msg);
        }),
        grounding_backup: grounding_backup.clone(),
    };
    let workflow_res = orch.run().await.map_err(|e: WorkflowError| e.0);
    timing_merge::merge_acp_with_grounding_restore(
        workflow_res,
        &artifacts.work_dir,
        &grounding_backup,
    )?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}


