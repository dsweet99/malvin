use malvin::acp::AgentClient;
use malvin::artifacts::{
    RunArtifacts, SessionDotfileBackups, backup_workspace_kissconfig_if_present,
    backup_workspace_kissignore_if_present, backup_workspace_malvin_checks_if_present,
    create_run_artifacts_from_text, resolve_user_request,
};
use malvin::orchestrator::{Orchestrator, WorkflowConfig, WorkflowError, workflow_context};
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{PromptError, PromptStore};

use super::repo_checks::{RepoGateOutput, run_repo_workspace_gates};
use super::{CodeArgs, SharedOpts};
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

pub fn prepare_bug_prompt_store(workflow: WorkflowCliOptions) -> Result<PromptStore, String> {
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
    let mut client = build_agent(shared, workflow, emit_stdout_markdown);
    let (text, work_dir) = resolve_user_request(&code.request)?;
    let artifacts = create_run_artifacts_from_text(&text, Some(work_dir.as_path()))
        .map_err(|e| e.to_string())?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
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
        log_full_outgoing_prompts: shared.verbose,
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
    super::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    let r = async {
        if !code.skip_pre_checks {
            run_repo_workspace_gates(
                &artifacts.work_dir,
                RepoGateOutput::Tagged,
                Some(&artifacts.run_dir),
            )?;
        }
        client.ensure_authenticated().map_err(|e| e.to_string())?;
        let ctx = workflow_context(&artifacts, &store, "code").map_err(|e: PromptError| e.0)?;
        let malvin_checks_backup = backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
        let kissconfig_backup = backup_workspace_kissconfig_if_present(&artifacts.work_dir)?;
        let kissignore_backup = backup_workspace_kissignore_if_present(&artifacts.work_dir)?;
        let session_dotfile_backups = SessionDotfileBackups::from_parts(
            kissconfig_backup.clone(),
            malvin_checks_backup.clone(),
            kissignore_backup.clone(),
        );
        run_emit::emit_run_startup_sequence(
            &artifacts,
            shared.tee_startup_stdout(),
            &code.request,
        )?;
        let workflow_res = {
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
                session_dotfile_backups: session_dotfile_backups.clone(),
            };
            orch.run_with_pre_summary_gap(
                &ctx,
                crate::cli::mid_session_gates::mid_pre_summary_repo_gates,
            )
            .await
            .map_err(|e: WorkflowError| e.0)
        };
        timing_merge::merge_acp_with_kissconfig_restore(
            workflow_res,
            &artifacts.work_dir,
            &session_dotfile_backups.kissconfig,
        )?;
        print_stdout_line(MALVIN_WHO, "DONE");
        Ok(())
    }
    .await;
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r
}
