//! Optional MPC planning agent session at the start of each outer gate-loop iteration when `mpc` is enabled (see `concepts_2.md` §5).

use std::path::{Path, PathBuf};

use crate::agent_backend::{build_agent_backend, AgentBackend};
use crate::mpc_planning_brief::MpcPlanningBriefAspect;
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use crate::prompts::{PromptError, PromptStore};
use crate::run_timing::TimingPhase;

use crate::acp::{CoderPromptOptions, restore_session_dotfiles};
use crate::kpop_progression::mpc_declared_done;

pub(crate) fn mpc_planner_iteration_log_path(artifacts: &RunArtifacts, iteration: usize) -> PathBuf {
    artifacts.log_path(&format!("mpc_planner_{iteration}"))
}

pub(crate) fn user_brief_declares_mpc_done(path: &Path) -> Result<bool, String> {
    let _aspect = MpcPlanningBriefAspect::DoneMarkerDetection;
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read user brief {}: {e}", path.display()))?;
    Ok(mpc_declared_done(&text))
}

pub(crate) fn mpc_enabled(work_dir: &Path) -> bool {
    let _aspect = MpcPlanningBriefAspect::ConfigEnabled;
    crate::malvin_config_file::load_malvin_config(work_dir).mpc
}

#[must_use]
pub(crate) fn mpc_planner_exp_log_path(artifacts: &RunArtifacts) -> PathBuf {
    let _aspect = MpcPlanningBriefAspect::HypothesisLogPath;
    artifacts.run_dir.join("_kpop").join("mpc_planner_log.md")
}

pub(crate) fn build_mpc_planner_context(
    base: &WorkflowRenderContext,
    artifacts: &RunArtifacts,
) -> WorkflowRenderContext {
    let mut ctx = base.clone();
    let exp_log_path = mpc_planner_exp_log_path(artifacts);
    let exp_log = crate::format_prompt_path(&exp_log_path, &artifacts.work_dir);
    ctx.insert("exp_log".to_string(), exp_log);
    ctx.insert(
        "current_state".to_string(),
        crate::current_state::format_current_state(
            artifacts.work_dir.as_path(),
            None,
            Some(artifacts),
        ),
    );
    ctx
}

/// Assemble `header.md` + `kpop_common.md` + `mpc_planner.md`.
///
/// # Errors
///
/// Returns `Err` when a prompt template cannot be rendered.
pub(crate) fn build_mpc_planner_prompt(
    store: &PromptStore,
    context: &WorkflowRenderContext,
) -> Result<String, String> {
    let _aspect = MpcPlanningBriefAspect::BriefAppendProtocol;
    let map = context.as_map();
    let header = store
        .render_prompt_only("header.md", map)
        .map_err(|e: PromptError| e.0)?;
    let common = store
        .render_prompt_only("kpop_common.md", map)
        .map_err(|e: PromptError| e.0)?;
    let body = store
        .render_prompt_only("mpc_planner.md", map)
        .map_err(|e: PromptError| e.0)?;
    Ok(join_labeled_strata([
        (PromptStratum::WorkflowHeader, header),
        (PromptStratum::EmbeddedTemplate, common),
        (PromptStratum::GateLoopBlock, body),
    ]))
}

pub(crate) struct MpcPlannerParams<'a> {
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub context: &'a WorkflowRenderContext,
    pub command: &'a str,
    pub client: Option<&'a mut AgentBackend>,
    /// Outer gate-loop iteration (1-based); suffixes `mpc_planner_{n}.log`.
    pub iteration: Option<usize>,
}

fn ensure_mpc_planner_exp_log(artifacts: &RunArtifacts) -> Result<PathBuf, String> {
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
) -> Result<(), String> {
    client
        .begin_coder_session(prepared.work_dir.as_path())
        .await
        .map_err(|e| e.to_string())?;
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
    restore_session_dotfiles(
        prepared.work_dir.as_path(),
        &prepared.session_dotfile_backups,
    )
    .map_err(|e| e.to_string())?;
    client
        .end_coder_session()
        .await
        .map_err(|e| e.to_string())?;
    prompt_result.map_err(|e| e.0)
}

fn mpc_planner_session_dotfiles(work_dir: &Path) -> Result<SessionDotfileBackups, String> {
    SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
}

/// Hook for [`super::run_loop::run_kpop_engine`]: optional MPC session at the start of one gate iteration.
pub(crate) async fn run_mpc_planner_for_kpop_engine_iteration(
    params: &super::params::KPopEngineParams<'_>,
    iteration: usize,
) -> Result<(), String> {
    let _aspect = MpcPlanningBriefAspect::PlannerSessionHook;
    run_mpc_planner_session(MpcPlannerParams {
        shared: params.shared,
        workflow: params.workflow,
        store: params.prepared.store(),
        artifacts: params.prepared.artifacts(),
        context: params.prepared.context(),
        command: params.command,
        client: None,
        iteration: Some(iteration),
    })
    .await
}

pub(crate) struct MpcPlannerTurnPrepared {
    pub(crate) prompt: String,
    pub(crate) work_dir: std::path::PathBuf,
    pub(crate) log_path: std::path::PathBuf,
    pub(crate) session_dotfile_backups: SessionDotfileBackups,
}

pub(crate) fn prepare_mpc_planner_turn(params: &MpcPlannerParams<'_>) -> Result<MpcPlannerTurnPrepared, String> {
    let _exp_log_path = ensure_mpc_planner_exp_log(params.artifacts)?;
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

/// Run one MPC planning-agent session when `mpc` is enabled in config.
///
/// # Errors
///
/// Returns `Err` when prompt assembly, agent I/O, or dotfile restore fails.
pub(crate) async fn run_mpc_planner_session(params: MpcPlannerParams<'_>) -> Result<(), String> {
    if !mpc_enabled(params.artifacts.work_dir.as_path()) {
        return Ok(());
    }
    let prepared = prepare_mpc_planner_turn(&params)?;
    if let Some(client) = params.client {
        client.set_prompts_log_run_dir(Some(params.artifacts.run_dir.clone()));
        return run_mpc_planner_with_client(client, &prepared).await;
    }

    let mut client = build_standalone_mpc_client(&params).await?;
    run_mpc_planner_with_client(&mut client, &prepared).await
}

#[cfg(test)]
#[path = "mpc_planner_tests.rs"]
mod mpc_planner_tests;
