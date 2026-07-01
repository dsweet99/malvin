//! MPC planning agent session at the start of each outer gate-loop iteration (see `concepts_2.md` §5).

use std::path::Path;

use crate::agent_backend::{build_agent_backend, AgentBackend};
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::PromptStore;
use crate::run_timing::TimingPhase;

use crate::acp::{CoderPromptOptions, restore_session_dotfiles};
use crate::session_dotfile_backup::{
    GitignoreBackup, KissConfigBackup, KissignoreBackup, MalvinChecksBackup,
    MalvinConfigBackup, MalvinConfigWorkspaceBackup, VisionBackup,
};

#[path = "mpc_planner_brief.rs"]
mod mpc_planner_brief;
#[allow(unused_imports)]
pub(crate) use mpc_planner_brief::{
    build_mpc_planner_context, build_mpc_planner_prompt, mpc_planner_exp_log_path,
    mpc_planner_iteration_log_path, reset_user_brief_before_planner, user_brief_baseline_path,
    user_brief_declares_mpc_done,
};

pub(crate) struct MpcPlannerParams<'a> {
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub context: &'a WorkflowRenderContext,
    pub command: &'a str,
    pub client: Option<&'a mut AgentBackend>,
    /// When true, leave the coder session open for a follow-on implementer prompt.
    pub keep_session_open: bool,
    /// Outer gate-loop iteration (1-based); suffixes `mpc_planner_{n}.log`.
    pub iteration: Option<usize>,
}

fn ensure_mpc_planner_exp_log(artifacts: &RunArtifacts) -> Result<std::path::PathBuf, String> {
    let exp_log_path = mpc_planner_exp_log_path(artifacts);
    if let Some(parent) = exp_log_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !exp_log_path.is_file() {
        std::fs::write(&exp_log_path, "").map_err(|e| e.to_string())?;
    }
    Ok(exp_log_path)
}

async fn run_mpc_planner_with_client(
    client: &mut AgentBackend,
    prepared: &MpcPlannerTurnPrepared,
    keep_session_open: bool,
) -> Result<(), String> {
    if !client.has_open_coder_session() {
        client
            .begin_coder_session(prepared.work_dir.as_path())
            .await
            .map_err(|e| e.to_string())?;
    }
    let prompt_result = client
        .run_coder_prompt(
            &prepared.prompt,
            prepared.log_path.as_path(),
            "mpc_planner",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                single_attempt: true,
                ..Default::default()
            },
        )
        .await;
    if !crate::acp::test_no_real_agent_enabled() {
        restore_session_dotfiles(
            prepared.work_dir.as_path(),
            &prepared.session_dotfile_backups,
        )
        .map_err(|e| e.to_string())?;
    }
    if !keep_session_open || prompt_result.is_err() {
        client
            .end_coder_session()
            .await
            .map_err(|e| e.to_string())?;
    }
    prompt_result.map_err(|e| e.0)
}

const fn mpc_planner_test_dotfile_backups() -> SessionDotfileBackups {
    SessionDotfileBackups {
        kissconfig: KissConfigBackup::Missing,
        malvin_checks: MalvinChecksBackup::Missing,
        kissignore: KissignoreBackup::Missing,
        malvin_config: MalvinConfigBackup::Missing,
        gitignore: GitignoreBackup::Missing,
        vision: VisionBackup::Missing,
        malvin_config_workspace: MalvinConfigWorkspaceBackup::Missing,
    }
}

fn mpc_planner_session_dotfiles(work_dir: &Path) -> Result<SessionDotfileBackups, String> {
    if crate::acp::test_no_real_agent_enabled() {
        crate::malvin_config_file::ensure_malvin_config_file_if_missing(work_dir)?;
        return Ok(mpc_planner_test_dotfile_backups());
    }
    SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
}

pub(crate) struct MpcPlannerTurnPrepared {
    pub(crate) prompt: String,
    pub(crate) work_dir: std::path::PathBuf,
    pub(crate) log_path: std::path::PathBuf,
    pub(crate) session_dotfile_backups: SessionDotfileBackups,
}

pub(crate) fn prepare_mpc_planner_turn(params: &MpcPlannerParams<'_>) -> Result<MpcPlannerTurnPrepared, String> {
    let _exp_log_path = ensure_mpc_planner_exp_log(params.artifacts)?;
    if !crate::acp::test_no_real_agent_enabled() {
        reset_user_brief_before_planner(params.artifacts, params.context)?;
    }
    let ctx = build_mpc_planner_context(params.context, params.artifacts);
    let prompt = build_mpc_planner_prompt(params.store, &ctx)?;
    let log_path = params.iteration.map_or_else(
        || params.artifacts.log_path("mpc_planner"),
        |iteration| mpc_planner_iteration_log_path(params.artifacts, iteration),
    );
    Ok(MpcPlannerTurnPrepared {
        work_dir: params.artifacts.work_dir.clone(),
        log_path,
        session_dotfile_backups: mpc_planner_session_dotfiles(params.artifacts.work_dir.as_path())?,
        prompt,
    })
}

async fn build_standalone_mpc_client(params: &MpcPlannerParams<'_>) -> Result<AgentBackend, String> {
    let mut client = build_agent_backend(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
        params.command,
    )
    .map_err(|e| e.to_string())?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    client.set_prompts_log_run_dir(Some(params.artifacts.run_dir.clone()));
    Ok(client)
}

/// Run one MPC planning-agent session.
///
/// # Errors
///
/// Returns `Err` when prompt assembly, agent I/O, or dotfile restore fails.
pub(crate) async fn run_mpc_planner_session(params: MpcPlannerParams<'_>) -> Result<(), String> {
    let prepared = prepare_mpc_planner_turn(&params)?;
    if let Some(client) = params.client {
        client.set_prompts_log_run_dir(Some(params.artifacts.run_dir.clone()));
        return run_mpc_planner_with_client(client, &prepared, params.keep_session_open).await;
    }

    let mut client = build_standalone_mpc_client(&params).await?;
    run_mpc_planner_with_client(&mut client, &prepared, params.keep_session_open).await
}

#[cfg(test)]
#[path = "mpc_planner_tests.rs"]
mod mpc_planner_tests;
