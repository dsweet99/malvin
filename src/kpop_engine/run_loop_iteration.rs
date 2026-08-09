use std::sync::{Arc, Mutex};

use crate::agent_backend::{agent_backend_set_run_timing, build_agent_backend};
use crate::cli::workflow_kpop_shared::kpop_engine_loop_iterations;

use super::super::kpop_session::{print_kpop_engine_log_line, run_kpop_engine_session, KPopEngineMultiturnCtx};
use super::super::params::{KPopEngineIterationParams, KPopEngineParams};
use super::{
    kpop_engine_gate_loop_early_exit, KPopEngineEarlyExitCtx, KPopEngineLoopOutcome,
};
use crate::artifacts::SessionDotfileBackups;

pub(crate) struct KpopEngineLoopIterationCtx<'a> {
    pub params: &'a super::super::params::KPopEngineParams<'a>,
    pub iteration: usize,
    pub run_timing: &'a Arc<Mutex<crate::run_timing::RunTiming>>,
    pub client: &'a mut crate::agent_backend::AgentBackend,
}

pub(crate) fn build_authenticated_kpop_engine_client(
    params: &KPopEngineParams<'_>,
    run_timing: &Arc<Mutex<crate::run_timing::RunTiming>>,
) -> Result<crate::agent_backend::AgentBackend, String> {
    let mut client = build_agent_backend(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
        params.command,
    )
    .map_err(|e| e.to_string())?;
    wire_kpop_engine_client(&mut client, params, run_timing);
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    Ok(client)
}

pub(crate) fn wire_kpop_engine_client(
    client: &mut crate::agent_backend::AgentBackend,
    params: &KPopEngineParams<'_>,
    run_timing: &Arc<Mutex<crate::run_timing::RunTiming>>,
) {
    agent_backend_set_run_timing(client, Some(Arc::clone(run_timing)));
    client.prompts_log_run_dir = Some(params.prepared.artifacts().run_dir.clone());
}

fn prepare_gate_iteration_artifacts(
    artifacts: &crate::artifacts::RunArtifacts,
    iteration: usize,
    work_dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    crate::session_dotfile_backup::repair_invalid_malvin_home_config_on_disk(work_dir)?;
    let exp_log_path =
        crate::artifacts::ensure_gate_exp_log_file(artifacts, iteration).map_err(|e| e.to_string())?;
    Ok(exp_log_path)
}

pub(crate) async fn run_kpop_engine_on_loop_iteration(
    ctx: KpopEngineLoopIterationCtx<'_>,
) -> Result<SessionDotfileBackups, String> {
    let params = ctx.params;
    let iteration = ctx.iteration;
    let client = ctx.client;
    let work_dir = &params.prepared.artifacts().work_dir;
    let exp_log_path =
        prepare_gate_iteration_artifacts(params.prepared.artifacts(), iteration, work_dir)?;

    let session_dotfile_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    print_kpop_engine_log_line(params.prepared, &exp_log_path);

    let total_iterations = kpop_engine_loop_iterations(params.max_loops);
    crate::gate_loop_session::set_active_gate_iteration(Some(iteration));
    let mut iteration_params = KPopEngineIterationParams {
        loop_params: params,
        session_dotfile_backups: &session_dotfile_backups,
        client,
        iteration,
        total_iterations,
        exp_log_path,
    };
    let mut ctx = KPopEngineMultiturnCtx {
        iteration: &mut iteration_params,
    };
    let post_agent_backups = run_kpop_engine_session(&mut ctx).await?;
    crate::gate_loop_session::set_active_gate_iteration(None);
    Ok(post_agent_backups)
}

pub(crate) async fn kpop_engine_loop_one_iteration(
    ctx: KpopEngineLoopIterationCtx<'_>,
    consecutive_solved: usize,
) -> Result<(usize, SessionDotfileBackups, Option<KPopEngineLoopOutcome>), String> {
    let params = ctx.params;
    let iteration = ctx.iteration;
    let run_timing = ctx.run_timing;
    let session_dotfile_backups = run_kpop_engine_on_loop_iteration(ctx).await?;
    let exp_log_path = params.prepared.artifacts().gate_exp_log_path(iteration);
    let streak = super::refresh_consecutive_solved_streak(
        consecutive_solved,
        &exp_log_path,
    )?;
    let early = kpop_engine_gate_loop_early_exit(KPopEngineEarlyExitCtx {
        behavior: params.behavior,
        consecutive_solved: streak,
        artifacts: params.prepared.artifacts(),
        session_dotfile_backups: &session_dotfile_backups,
        agent_ran: true,
        run_timing: Some(run_timing),
    });
    Ok((streak, session_dotfile_backups, early))
}
