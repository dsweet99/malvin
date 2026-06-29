use std::sync::{Arc, Mutex};

use crate::agent_backend::{agent_backend_set_run_timing, build_agent_backend};
use crate::cli::workflow_kpop_shared::kpop_engine_loop_iterations;

use super::super::kpop_session::{print_kpop_engine_log_line, run_kpop_engine_session, KPopEngineMultiturnCtx};
use super::super::params::{KPopEngineIterationParams, KPopEngineParams};
use super::{
    kpop_engine_solved_early_exit, refresh_consecutive_solved_streak, KPopEngineEarlyExitCtx,
    KPopEngineLoopOutcome, KpopEngineLoopIterationCtx,
};
use crate::artifacts::SessionDotfileBackups;

fn build_authenticated_kpop_engine_client(
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
    client.set_prompts_log_run_dir(Some(params.prepared.artifacts().run_dir.clone()));
}

pub(crate) async fn run_kpop_engine_on_loop_iteration(
    params: &KPopEngineParams<'_>,
    iteration: usize,
    run_timing: &Arc<Mutex<crate::run_timing::RunTiming>>,
    consecutive_solved_entering: usize,
) -> Result<SessionDotfileBackups, String> {
    let work_dir = &params.prepared.artifacts().work_dir;
    crate::session_dotfile_backup::repair_clamp_damaged_dotfiles_on_disk(work_dir)?;
    let exp_log_path = crate::artifacts::ensure_gate_exp_log_file(
        params.prepared.artifacts(),
        iteration,
    )
    .map_err(|e| e.to_string())?;

    let mut client = build_authenticated_kpop_engine_client(params, run_timing)?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    print_kpop_engine_log_line(params.prepared, &exp_log_path);

    let total_iterations = kpop_engine_loop_iterations(params.max_loops);
    crate::gate_loop_session::set_active_gate_iteration(Some(iteration));
    let mut iteration_params = KPopEngineIterationParams {
        loop_params: params,
        session_dotfile_backups: &session_dotfile_backups,
        client: &mut client,
        iteration,
        total_iterations,
        consecutive_solved_entering,
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
) -> Result<(usize, SessionDotfileBackups, Option<KPopEngineLoopOutcome>), String> {
    let session_dotfile_backups = run_kpop_engine_on_loop_iteration(
        ctx.params,
        ctx.iteration,
        ctx.run_timing,
        ctx.consecutive_solved,
    )
    .await?;
    let exp_log_path = ctx.params.prepared.artifacts().gate_exp_log_path(ctx.iteration);
    let streak = refresh_consecutive_solved_streak(ctx.consecutive_solved, &exp_log_path)?;
    let early = kpop_engine_solved_early_exit(KPopEngineEarlyExitCtx {
        behavior: ctx.params.behavior,
        consecutive_solved: streak,
        artifacts: ctx.params.prepared.artifacts(),
        session_dotfile_backups: &session_dotfile_backups,
        agent_ran: true,
        run_timing: Some(ctx.run_timing),
        mpc_enabled: ctx.mpc_on,
    });
    Ok((streak, session_dotfile_backups, early))
}
