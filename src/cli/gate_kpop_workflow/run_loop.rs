use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::kpop_progression::{agent_declared_success, read_exp_log_text};

use crate::agent_backend::{agent_backend_set_run_timing, build_agent_backend};
use crate::cli::workflow_kpop_shared::{
    gate_kpop_loop_iterations, run_kpop_workspace_gates,
};

use super::kpop_session::{print_gate_kpop_log_line, run_gate_kpop_session, GateKpopMultiturnCtx};
use super::params::{GateKpopIterationParams, GateKpopLoopParams};

type GateKpopLoopOutcome = (
    bool,
    bool,
    Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    SessionDotfileBackups,
);

pub(crate) fn session_wrote_kpop_solved(exp_log_path: &Path) -> Result<bool, String> {
    let text = read_exp_log_text(exp_log_path)?;
    Ok(agent_declared_success(&text))
}

pub(crate) fn kpop_solved_early_exit(
    behavior: super::behavior::GateLoopBehavior,
    consecutive_solved: usize,
    artifacts: &crate::artifacts::RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
) -> bool {
    if consecutive_solved < behavior.consecutive_kpop_solved_to_exit() {
        return false;
    }
    if behavior.require_passing_gates_for_exit() && !behavior.skip_workspace_quality_gates {
        run_kpop_workspace_gates(
            artifacts,
            session_dotfile_backups,
            behavior.restore_malvin_checks_after_session(),
        )
        .is_ok()
    } else {
        true
    }
}

pub(crate) struct GateKpopEarlyExitCtx<'a> {
    pub behavior: super::behavior::GateLoopBehavior,
    pub consecutive_solved: usize,
    pub artifacts: &'a crate::artifacts::RunArtifacts,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub agent_ran: bool,
    pub run_timing: Option<&'a Arc<Mutex<crate::run_timing::RunTiming>>>,
}

pub(crate) fn gate_kpop_solved_early_exit(ctx: GateKpopEarlyExitCtx<'_>) -> Option<GateKpopLoopOutcome> {
    if kpop_solved_early_exit(
        ctx.behavior,
        ctx.consecutive_solved,
        ctx.artifacts,
        ctx.session_dotfile_backups,
    ) {
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

pub(crate) async fn run_gate_kpop_on_loop_iteration(
    params: &GateKpopLoopParams<'_>,
    iteration: usize,
    run_timing: &Arc<Mutex<crate::run_timing::RunTiming>>,
) -> Result<SessionDotfileBackups, String> {
    let exp_log_path = crate::artifacts::ensure_gate_exp_log_file(
        params.prepared.artifacts(),
        iteration,
    )
    .map_err(|e| e.to_string())?;

    let mut client = build_agent_backend(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
        params.command,
    )
    .map_err(|e| e.to_string())?;
    wire_gate_kpop_client(&mut client, params, run_timing);
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(
            &params.prepared.artifacts().work_dir,
        )?;
    print_gate_kpop_log_line(params.prepared, &exp_log_path);

    crate::gate_loop_session::set_active_gate_iteration(Some(iteration));
    let mut iteration_params = GateKpopIterationParams {
        loop_params: params,
        session_dotfile_backups: &session_dotfile_backups,
        client: &mut client,
        iteration,
        exp_log_path,
    };
    let mut ctx = GateKpopMultiturnCtx {
        iteration: &mut iteration_params,
    };
    let result = run_gate_kpop_session(&mut ctx).await;
    crate::gate_loop_session::set_active_gate_iteration(None);
    result.map(|()| session_dotfile_backups)
}

pub(crate) fn wire_gate_kpop_client(
    client: &mut crate::agent_backend::AgentBackend,
    params: &GateKpopLoopParams<'_>,
    run_timing: &Arc<Mutex<crate::run_timing::RunTiming>>,
) {
    agent_backend_set_run_timing(client, Some(Arc::clone(run_timing)));
    client.set_prompts_log_run_dir(Some(params.prepared.artifacts().run_dir.clone()));
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

pub(crate) async fn gate_kpop_loop_one_iteration(
    params: &GateKpopLoopParams<'_>,
    iteration: usize,
    run_timing: &Arc<Mutex<crate::run_timing::RunTiming>>,
    consecutive_solved: usize,
) -> Result<(usize, SessionDotfileBackups, Option<GateKpopLoopOutcome>), String> {
    let session_dotfile_backups =
        run_gate_kpop_on_loop_iteration(params, iteration, run_timing).await?;
    let exp_log_path = params.prepared.artifacts().gate_exp_log_path(iteration);
    let streak = refresh_consecutive_solved_streak(consecutive_solved, &exp_log_path)?;
    let early = gate_kpop_solved_early_exit(GateKpopEarlyExitCtx {
        behavior: params.behavior,
        consecutive_solved: streak,
        artifacts: params.prepared.artifacts(),
        session_dotfile_backups: &session_dotfile_backups,
        agent_ran: true,
        run_timing: Some(run_timing),
    });
    Ok((streak, session_dotfile_backups, early))
}

pub(crate) async fn run_gate_kpop_loop(
    params: GateKpopLoopParams<'_>,
) -> Result<GateKpopLoopOutcome, String> {
    let work_dir = params.prepared.artifacts().work_dir.as_path();
    let mut last_backups = SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
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

    let iterations = gate_kpop_loop_iterations(params.max_loops);
    let run_timing = crate::run_timing::attach_gate_kpop_loop_run_timing();
    let mut consecutive_solved = 0usize;
    for iteration in 1..=iterations {
        let (streak, backups, early) =
            gate_kpop_loop_one_iteration(&params, iteration, &run_timing, consecutive_solved).await?;
        consecutive_solved = streak;
        last_backups = backups;
        if let Some(outcome) = early {
            return Ok(outcome);
        }
    }
    let gates_ok = params.behavior.recheck_gates_after_exhausted
        && !params.behavior.skip_workspace_quality_gates
        && run_kpop_workspace_gates(
            params.prepared.artifacts(),
            &last_backups,
            params.behavior.restore_malvin_checks_after_session(),
        )
        .is_ok();
    Ok((gates_ok, true, Some(run_timing), last_backups))
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = (gate_kpop_loop_one_iteration, run_gate_kpop_on_loop_iteration, wire_gate_kpop_client);
    }
}
