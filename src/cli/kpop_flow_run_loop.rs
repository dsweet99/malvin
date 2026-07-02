//! Outer `malvin kpop` agent loop (`--max-loops`, early exit on `DONE` in mpc plan file).

use std::path::PathBuf;

use crate::artifacts::{ensure_gate_exp_log_file, SessionDotfileBackups};
use crate::cli::KpopArgs;
use crate::kpop_progression::{mpc_plan_declares_done, KpopMultiturnState};
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::prompts::PromptStore;
use crate::KpopTurnPrompts;

use super::kpop_flow_a::{kpop_run_acp_multiturn, KpopAcpMultiturnCtx};
use super::KpopPrepared;
use crate::cli::loop_opts::kpop_agent_loop_exp_iteration;
use crate::cli::workflow_kpop_shared::{
    effective_max_loops, gate_iteration_context, print_kpop_session_log_line,
};

pub(crate) struct RunKpopAgentLoopsParams<'a> {
    pub kpop: &'a KpopArgs,
    pub store: &'a PromptStore,
    pub client: &'a mut crate::agent_backend::AgentBackend,
    pub prepared: &'a KpopPrepared,
}

pub(crate) struct RunKpopAgentLoopsOutcome {
    pub acp_result: Result<(), String>,
    /// True when at least one outer-loop iteration invoked the kpop agent.
    pub agent_ran: bool,
}

pub(crate) const fn kpop_loop_abort(agent_ran: bool, err: String) -> RunKpopAgentLoopsOutcome {
    RunKpopAgentLoopsOutcome {
        acp_result: Err(err),
        agent_ran,
    }
}

pub(crate) struct KpopLoopSnapshot {
    pub backups: SessionDotfileBackups,
    pub exp_iter: usize,
    pub exp_log_path: PathBuf,
}

pub(crate) fn snapshot_kpop_loop_dotfiles_and_exp_log(
    artifacts: &crate::artifacts::RunArtifacts,
    agent_loop: usize,
    max_loops: usize,
) -> Result<KpopLoopSnapshot, String> {
    let backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(&artifacts.work_dir)?;
    let exp_iter = kpop_agent_loop_exp_iteration(agent_loop, max_loops);
    let exp_log_path = ensure_gate_exp_log_file(artifacts, exp_iter).map_err(|e| e.to_string())?;
    crate::artifacts::reset_mpc_plan_file_for_iteration(artifacts, agent_loop)
        .map_err(|e| e.to_string())?;
    Ok(KpopLoopSnapshot {
        backups,
        exp_iter,
        exp_log_path,
    })
}

pub(crate) struct KpopLoopExitAfterIteration {
    pub(crate) will_exit_after_this_loop: bool,
    pub(crate) early_exit_on_solved: bool,
}

pub(crate) fn kpop_loop_exit_after_iteration(
    artifacts: &crate::artifacts::RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
    agent_loop: usize,
    max_loops: usize,
) -> Result<KpopLoopExitAfterIteration, String> {
    let mpc_plan_done =
        mpc_plan_declares_done(&crate::artifacts::mpc_plan_path(artifacts))?;
    let gate_ctx = crate::kpop_engine::GateLoopExitCtx {
        behavior: crate::kpop_engine::KPopHardConstraints::CODE,
        artifacts,
        session_dotfile_backups,
    };
    let early_exit = crate::kpop_engine::mpc_plan_early_exit(&gate_ctx, mpc_plan_done);
    Ok(KpopLoopExitAfterIteration {
        will_exit_after_this_loop: early_exit || agent_loop == max_loops,
        early_exit_on_solved: early_exit,
    })
}

async fn finish_kpop_loop_iteration(
    params: &mut RunKpopAgentLoopsParams<'_>,
    loop_snapshot: &KpopLoopSnapshot,
    bounds: (usize, usize),
) -> Result<Option<bool>, String> {
    let (agent_loop, max_loops) = bounds;
    let exit = kpop_loop_exit_after_iteration(
        &params.prepared.artifacts,
        &loop_snapshot.backups,
        agent_loop,
        max_loops,
    )?;
    crate::cli::kpop_summarize::maybe_run_inline_summarize_on_kpop_loop(
        crate::cli::kpop_summarize::InlineSummarizeOnKpopLoopCtx {
            client: params.client,
            store: params.store,
            artifacts: &params.prepared.artifacts,
            agent_loop,
            max_loops,
            will_exit_after_this_loop: exit.will_exit_after_this_loop,
        },
    )
    .await?;
    Ok(exit.early_exit_on_solved.then_some(true))
}

async fn run_kpop_multiturn_for_loop(
    params: &mut RunKpopAgentLoopsParams<'_>,
    loop_snapshot: &KpopLoopSnapshot,
    agent_loop: usize,
    max_loops: usize,
) -> Result<Result<(), String>, String> {
    print_kpop_session_log_line(&params.prepared.artifacts, &loop_snapshot.exp_log_path);
    let iteration_context = gate_iteration_context(
        &params.prepared.context,
        &params.prepared.artifacts,
        &loop_snapshot.exp_log_path,
        loop_snapshot.exp_iter,
    );
    let builder = KpopMultiturnPrompts::Turn(KpopTurnPrompts {
        store: params.store,
        base: &iteration_context,
        prepend_rules_once: agent_loop == 1,
    });
    let mut state = KpopMultiturnState::new(
        builder,
        loop_snapshot.exp_log_path.clone(),
        crate::artifacts::mpc_plan_path(&params.prepared.artifacts),
        params.kpop.max_hypotheses,
    )?;
    crate::gate_loop_session::set_active_gate_iteration(Some(loop_snapshot.exp_iter));
    let acp_result = kpop_run_acp_multiturn(
        KpopAcpMultiturnCtx {
            client: params.client,
            prepared: params.prepared,
            state: &mut state,
        },
        &loop_snapshot.backups,
        if agent_loop == max_loops {
            crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize
        } else {
            crate::run_timing::acp_post_run::RunTimingSessionEnd::AccumulateRun
        },
    )
    .await;
    crate::gate_loop_session::set_active_gate_iteration(None);
    Ok(acp_result)
}

async fn run_kpop_agent_loop_turn(
    params: &mut RunKpopAgentLoopsParams<'_>,
    agent_loop: usize,
    max_loops: usize,
) -> Result<(Option<RunKpopAgentLoopsOutcome>, Result<(), String>), String> {
    let loop_snapshot = snapshot_kpop_loop_dotfiles_and_exp_log(
        &params.prepared.artifacts,
        agent_loop,
        max_loops,
    )?;
    let last_acp = run_kpop_multiturn_for_loop(params, &loop_snapshot, agent_loop, max_loops).await?;
    if last_acp.is_err() {
        return Ok((
            Some(RunKpopAgentLoopsOutcome {
                acp_result: last_acp,
                agent_ran: true,
            }),
            Ok(()),
        ));
    }
    if finish_kpop_loop_iteration(params, &loop_snapshot, (agent_loop, max_loops))
        .await?
        .is_some()
    {
        return Ok((
            Some(RunKpopAgentLoopsOutcome {
                acp_result: last_acp,
                agent_ran: true,
            }),
            Ok(()),
        ));
    }
    Ok((None, last_acp))
}

pub(crate) async fn run_kpop_agent_loops(
    mut params: RunKpopAgentLoopsParams<'_>,
) -> RunKpopAgentLoopsOutcome {
    let max_loops = effective_max_loops(params.kpop.max_loops);
    clear_legacy_gate_exp_log(&params.prepared.artifacts, max_loops);
    let mut last_acp = Ok(());
    let mut agent_ran = false;
    for agent_loop in 1..=max_loops {
        agent_ran = true;
        match run_kpop_agent_loop_turn(&mut params, agent_loop, max_loops).await {
            Ok((Some(outcome), _)) => return outcome,
            Ok((None, acp)) => last_acp = acp,
            Err(e) => return kpop_loop_abort(agent_ran, e),
        }
    }
    RunKpopAgentLoopsOutcome {
        acp_result: last_acp,
        agent_ran,
    }
}

pub(crate) fn clear_legacy_gate_exp_log(artifacts: &crate::artifacts::RunArtifacts, max_loops: usize) {
    if max_loops > 1 {
        let _ = std::fs::remove_file(artifacts.gate_exp_log_path(0));
    }
}
