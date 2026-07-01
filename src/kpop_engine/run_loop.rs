use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::kpop_progression::{agent_declared_success, read_exp_log_text};
use crate::mpc_planning_brief::MpcPlanningBriefAspect;

use crate::cli::workflow_kpop_shared::{
    kpop_engine_loop_iterations, run_kpop_workspace_gates,
};

pub(crate) use super::run_loop_exit::{
    mpc_done_early_exit, run_gate_workspace_gates_with_fresh_backups, GateLoopExitCtx,
};

#[path = "run_loop_iteration.rs"]
mod run_loop_iteration;
pub(crate) use run_loop_iteration::{kpop_engine_loop_one_iteration, KpopEngineLoopIterationCtx};
#[cfg(test)]
pub(crate) use run_loop_iteration::{run_kpop_engine_on_loop_iteration, wire_kpop_engine_client};

pub(crate) type KPopEngineLoopOutcome = (
    bool,
    bool,
    Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    SessionDotfileBackups,
);

pub(crate) fn session_wrote_kpop_solved(exp_log_path: &Path) -> Result<bool, String> {
    let text = read_exp_log_text(exp_log_path)?;
    Ok(agent_declared_success(&text))
}

/// Restore loop-carried dotfile backups before anchoring the next iteration.
///
/// Without this, a fail path that leaves disk regressed poisons the next iteration's
/// pre-agent snapshot even when the in-memory merged bundle is still sane.
pub(crate) fn restore_carry_forward_before_iteration_snapshot(
    work_dir: &Path,
    carry_forward: Option<&SessionDotfileBackups>,
) -> Result<(), String> {
    if let Some(prior) = carry_forward {
        let mut sanitized = prior.clone();
        crate::session_dotfile_backup::sanitize_clamp_damaged_dotfiles_in_bundle(
            &mut sanitized,
            work_dir,
        );
        sanitized.restore(work_dir)?;
    }
    Ok(())
}

use crate::artifacts::SessionDotfileBackups;

pub(crate) fn refresh_consecutive_solved_streak(
    consecutive_solved: usize,
    exp_log_path: &Path,
) -> Result<usize, String> {
    if session_wrote_kpop_solved(exp_log_path)? {
        Ok(consecutive_solved.saturating_add(1))
    } else {
        Ok(0)
    }
}

async fn mpc_done_exit_after_planner(
    params: &super::params::KPopEngineParams<'_>,
    iteration: usize,
    last_backups: &SessionDotfileBackups,
    client: &mut crate::agent_backend::AgentBackend,
) -> Result<bool, String> {
    let _aspect = MpcPlanningBriefAspect::PlannerSessionHook;
    super::mpc_planner::run_mpc_planner_session(super::mpc_planner::MpcPlannerParams {
        shared: params.shared,
        workflow: params.workflow,
        store: params.prepared.store(),
        artifacts: params.prepared.artifacts(),
        context: params.prepared.context(),
        command: params.command,
        client: Some(client),
        keep_session_open: true,
        iteration: Some(iteration),
    })
    .await?;
    let brief_path = crate::workflow_context::resolve_user_brief_path(
        params.prepared.artifacts(),
        params.prepared.context(),
    );
    let gate_ctx = GateLoopExitCtx {
        behavior: params.behavior,
        artifacts: params.prepared.artifacts(),
        session_dotfile_backups: last_backups,
    };
    mpc_done_early_exit(&gate_ctx, &brief_path)
}

fn exhausted_loop_gate_ok(
    params: &super::params::KPopEngineParams<'_>,
    last_backups: &SessionDotfileBackups,
) -> bool {
    params.behavior.recheck_gates_after_exhausted
        && !params.behavior.skip_workspace_quality_gates
        && run_gate_workspace_gates_with_fresh_backups(
            params.prepared.artifacts(),
            last_backups,
            params.behavior,
        )
}

fn prepare_kpop_engine_loop(work_dir: &Path) -> Result<SessionDotfileBackups, String> {
    crate::session_dotfile_backup::repair_clamp_damaged_dotfiles_on_disk(work_dir)?;
    SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
}

struct KpopEngineIterationInput<'a> {
    params: &'a super::params::KPopEngineParams<'a>,
    iteration: usize,
    consecutive_solved: usize,
    last_backups: &'a SessionDotfileBackups,
    run_timing: &'a Arc<Mutex<crate::run_timing::RunTiming>>,
}

async fn run_kpop_engine_iteration(
    input: KpopEngineIterationInput<'_>,
) -> Result<(usize, SessionDotfileBackups, bool), String> {
    let KpopEngineIterationInput {
        params,
        iteration,
        consecutive_solved,
        last_backups,
        run_timing,
    } = input;
    let mut client =
        run_loop_iteration::build_authenticated_kpop_engine_client(params, run_timing)?;
    if mpc_done_exit_after_planner(params, iteration, last_backups, &mut client).await? {
        if client.has_open_coder_session() {
            client.end_coder_session().await.map_err(|e| e.to_string())?;
        }
        return Ok((consecutive_solved, last_backups.clone(), true));
    }
    let (streak, backups) = kpop_engine_loop_one_iteration(KpopEngineLoopIterationCtx {
        params,
        iteration,
        consecutive_solved,
        client: &mut client,
    })
    .await?;
    if client.has_open_coder_session() {
        client.end_coder_session().await.map_err(|e| e.to_string())?;
    }
    Ok((streak, backups, false))
}

pub(crate) async fn run_kpop_engine(
    params: super::params::KPopEngineParams<'_>,
) -> Result<KPopEngineLoopOutcome, String> {
    let work_dir = params.prepared.artifacts().work_dir.as_path();
    let mut last_backups = prepare_kpop_engine_loop(work_dir)?;
    if params.behavior.skip_kpop_on_initial_pass
        && !params.behavior.skip_workspace_quality_gates
        && run_kpop_workspace_gates(
            params.prepared.artifacts(),
            &last_backups,
            params.behavior.restore_malvin_checks_after_session(),
        )
        .is_ok()
    {
        return Ok((true, false, None, last_backups));
    }

    let iterations = kpop_engine_loop_iterations(params.max_loops);
    let run_timing = crate::run_timing::attach_kpop_engine_loop_run_timing();
    let mut consecutive_solved = 0usize;
    for iteration in 1..=iterations {
        if iteration > 1 {
            restore_carry_forward_before_iteration_snapshot(work_dir, Some(&last_backups))?;
        }
        let (streak, backups, mpc_done) = run_kpop_engine_iteration(KpopEngineIterationInput {
            params: &params,
            iteration,
            consecutive_solved,
            last_backups: &last_backups,
            run_timing: &run_timing,
        })
        .await?;
        if mpc_done {
            return Ok((true, false, Some(run_timing), last_backups));
        }
        consecutive_solved = streak;
        last_backups = backups;
    }
    Ok((exhausted_loop_gate_ok(&params, &last_backups), true, Some(run_timing), last_backups))
}

#[cfg(test)]
#[path = "run_loop_mpc_tests.rs"]
mod run_loop_mpc_tests;

#[cfg(test)]
#[path = "run_loop_tests.rs"]
pub(crate) mod run_loop_tests;
