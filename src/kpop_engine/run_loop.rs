use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::cli::workflow_kpop_shared::{
    kpop_engine_loop_iterations, run_kpop_workspace_gates,
};

pub(crate) use super::run_loop_exit::{
    gate_loop_early_exit, kpop_solved_early_exit, refresh_consecutive_solved_streak,
    run_gate_workspace_gates_with_fresh_backups, GateLoopExitCtx,
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

pub(crate) struct KPopEngineEarlyExitCtx<'a> {
    pub behavior: super::behavior::KPopHardConstraints,
    pub consecutive_solved: usize,
    pub artifacts: &'a crate::artifacts::RunArtifacts,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub agent_ran: bool,
    pub run_timing: Option<&'a Arc<Mutex<crate::run_timing::RunTiming>>>,
}

pub(crate) fn kpop_engine_gate_loop_early_exit(
    ctx: KPopEngineEarlyExitCtx<'_>,
) -> Option<KPopEngineLoopOutcome> {
    if kpop_solved_early_exit(
        ctx.behavior,
        ctx.consecutive_solved,
        ctx.artifacts,
        ctx.session_dotfile_backups,
    ) {
        return Some((
            true,
            ctx.agent_ran,
            ctx.run_timing.cloned(),
            ctx.session_dotfile_backups.clone(),
        ));
    }
    let gate_ctx = GateLoopExitCtx {
        behavior: ctx.behavior,
        artifacts: ctx.artifacts,
        session_dotfile_backups: ctx.session_dotfile_backups,
    };
    if gate_loop_early_exit(&gate_ctx) {
        Some((
            true,
            ctx.agent_ran,
            ctx.run_timing.cloned(),
            ctx.session_dotfile_backups.clone(),
        ))
    } else {
        None
    }
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
        crate::session_dotfile_backup::sanitize_invalid_malvin_home_config_in_bundle(
            &mut sanitized,
            work_dir,
        );
        sanitized.restore(work_dir)?;
    }
    Ok(())
}

use crate::artifacts::SessionDotfileBackups;

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
    crate::session_dotfile_backup::repair_invalid_malvin_home_config_on_disk(work_dir)?;
    SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
}

struct KpopEngineIterationInput<'a> {
    params: &'a super::params::KPopEngineParams<'a>,
    iteration: usize,
    run_timing: &'a Arc<Mutex<crate::run_timing::RunTiming>>,
    consecutive_solved: usize,
}

async fn run_kpop_engine_iteration(
    input: KpopEngineIterationInput<'_>,
) -> Result<(usize, SessionDotfileBackups, Option<KPopEngineLoopOutcome>), String> {
    let KpopEngineIterationInput {
        params,
        iteration,
        run_timing,
        consecutive_solved,
    } = input;
    let mut client =
        run_loop_iteration::build_authenticated_kpop_engine_client(params, run_timing)?;
    let (streak, backups, early) = kpop_engine_loop_one_iteration(
        KpopEngineLoopIterationCtx {
            params,
            iteration,
            run_timing,
            client: &mut client,
        },
        consecutive_solved,
    )
    .await?;
    if client.has_open_coder_session() {
        client.end_coder_session().await.map_err(|e| e.to_string())?;
    }
    Ok((streak, backups, early))
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
        let (streak, backups, early) = run_kpop_engine_iteration(KpopEngineIterationInput {
            params: &params,
            iteration,
            run_timing: &run_timing,
            consecutive_solved,
        })
        .await?;
        consecutive_solved = streak;
        last_backups = backups;
        if let Some(outcome) = early {
            return Ok(outcome);
        }
    }
    Ok((exhausted_loop_gate_ok(&params, &last_backups), true, Some(run_timing), last_backups))
}

#[cfg(test)]
#[path = "run_loop_exit_tests.rs"]
mod run_loop_exit_tests;

#[cfg(test)]
#[path = "run_loop_tests.rs"]
pub(crate) mod run_loop_tests;
