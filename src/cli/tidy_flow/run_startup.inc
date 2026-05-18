use std::collections::HashMap;
use std::path::Path;

use malvin::artifacts::{
    MalvinChecksBackup, RunArtifacts, SessionDotfileBackups,
    backup_workspace_kissconfig_if_present,
    backup_workspace_kissignore_if_present,
    backup_workspace_malvin_checks_if_present,
    create_run_artifacts_from_text,
};
use malvin::orchestrator::workflow_context;
use malvin::prompts::{PromptError, PromptStore};

use crate::cli::repo_checks::{RepoGateOutput, run_repo_workspace_gates};
use crate::cli::{SharedOpts, WorkflowCliOptions, build_agent, emit_run_startup_sequence};

use super::prep::prepare_tidy_prompt_store;
use super::TidyStartup;

pub fn tidy_prompt_context(
    artifacts: &RunArtifacts,
) -> Result<(PromptStore, HashMap<String, String>), String> {
    let store = prepare_tidy_prompt_store()?;
    let context = workflow_context(artifacts, &store, "tidy").map_err(|e: PromptError| e.0)?;
    Ok((store, context))
}

fn tidy_session_dotfile_backups(
    work_dir: &Path,
    malvin_checks_backup: MalvinChecksBackup,
) -> Result<SessionDotfileBackups, String> {
    let kissconfig_backup = backup_workspace_kissconfig_if_present(work_dir)?;
    let kissignore_backup = backup_workspace_kissignore_if_present(work_dir)?;
    Ok(SessionDotfileBackups::from_parts(
        kissconfig_backup,
        malvin_checks_backup,
        kissignore_backup,
    ))
}

fn tidy_skip_agent_startup(
    artifacts: RunArtifacts,
    shared: &SharedOpts,
    malvin_checks_backup: MalvinChecksBackup,
) -> Result<TidyStartup, String> {
    emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), "tidy")?;
    let session_dotfile_backups =
        tidy_session_dotfile_backups(&artifacts.work_dir, malvin_checks_backup)?;
    Ok(TidyStartup::SkipAgent {
        artifacts,
        session_dotfile_backups,
    })
}

pub(super) struct TidyAgentStartupRequest<'a> {
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub artifacts: RunArtifacts,
    pub malvin_checks_backup: MalvinChecksBackup,
    pub run_learn: bool,
}

fn tidy_run_agent_startup(req: TidyAgentStartupRequest<'_>) -> Result<TidyStartup, String> {
    let mut client = build_agent(req.shared, req.workflow, req.shared.acp_stdout_markdown_enabled());
    client.prompts_log_run_dir = Some(req.artifacts.run_dir.clone());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    emit_run_startup_sequence(&req.artifacts, req.shared.tee_startup_stdout(), "tidy")?;
    let session_dotfile_backups =
        tidy_session_dotfile_backups(&req.artifacts.work_dir, req.malvin_checks_backup)?;
    let (store, context) = tidy_prompt_context(&req.artifacts)?;
    Ok(TidyStartup::RunAgent {
        client,
        artifacts: req.artifacts,
        session_dotfile_backups,
        store,
        context,
        run_learn: req.run_learn,
    })
}

pub fn prepare_tidy_run(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    run_learn: bool,
) -> Result<TidyStartup, String> {
    let artifacts =
        create_run_artifacts_from_text("tidy", Some(Path::new("."))).map_err(|e| e.to_string())?;
    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;

    let gates_ok = run_repo_workspace_gates(
        &artifacts.work_dir,
        RepoGateOutput::Tagged,
        Some(&artifacts.run_dir),
    )
    .is_ok();

    if gates_ok {
        return tidy_skip_agent_startup(artifacts, shared, malvin_checks_backup);
    }

    tidy_run_agent_startup(TidyAgentStartupRequest {
        shared,
        workflow,
        artifacts,
        malvin_checks_backup,
        run_learn,
    })
}
