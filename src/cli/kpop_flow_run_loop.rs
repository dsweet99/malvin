//! Outer `malvin kpop` agent loop (`--max-loops`, early exit on `## KPOP_SOLVED`).

mod kpop_flow_run_loop_types;
pub(crate) use kpop_flow_run_loop_types::RunKpopAgentLoopsParams;

use std::path::PathBuf;

use crate::artifacts::{ensure_gate_exp_log_file, SessionDotfileBackups};
use crate::kpop_progression::{agent_declared_success, read_exp_log_text, KpopMultiturnState};
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::KpopTurnPrompts;

use super::kpop_flow_a::{kpop_run_acp_multiturn, KpopAcpMultiturnCtx};
use crate::cli::loop_opts::kpop_agent_loop_exp_iteration;
use crate::cli::workflow_kpop_shared::{
    effective_max_loops, gate_iteration_context, print_kpop_session_log_line,
};

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
    Ok(KpopLoopSnapshot {
        backups,
        exp_iter,
        exp_log_path,
    })
}

struct KpopLoopExitAfterIteration {
    declares_solved: bool,
    will_exit_after_this_loop: bool,
}

fn kpop_loop_exit_after_iteration(
    exp_log_path: &PathBuf,
    agent_loop: usize,
    max_loops: usize,
) -> Result<KpopLoopExitAfterIteration, String> {
    let declares_solved = kpop_exp_log_declares_solved(exp_log_path)?;
    Ok(KpopLoopExitAfterIteration {
        declares_solved,
        will_exit_after_this_loop: declares_solved || agent_loop == max_loops,
    })
}

async fn finish_kpop_loop_iteration(
    params: &mut RunKpopAgentLoopsParams<'_>,
    loop_snapshot: &KpopLoopSnapshot,
    agent_loop: usize,
    max_loops: usize,
) -> Result<Option<bool>, String> {
    let exit = kpop_loop_exit_after_iteration(
        &loop_snapshot.exp_log_path,
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
    Ok(exit.declares_solved.then_some(true))
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
        let loop_snapshot = match snapshot_kpop_loop_dotfiles_and_exp_log(
            &params.prepared.artifacts,
            agent_loop,
            max_loops,
        ) {
            Ok(s) => s,
            Err(e) => return kpop_loop_abort(agent_ran, e),
        };
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
            request_text: &params.prepared.text,
            prepend_rules_once: agent_loop == 1,
        });
        let mut state = match KpopMultiturnState::new(
            builder,
            loop_snapshot.exp_log_path.clone(),
            params.kpop.max_hypotheses,
        ) {
            Ok(s) => s,
            Err(e) => return kpop_loop_abort(agent_ran, e),
        };
        crate::gate_loop_session::set_active_gate_iteration(Some(loop_snapshot.exp_iter));
        last_acp = kpop_run_acp_multiturn(
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
        if last_acp.is_err() {
            break;
        }
        match finish_kpop_loop_iteration(&mut params, &loop_snapshot, agent_loop, max_loops).await {
            Ok(Some(_)) => break,
            Ok(None) => {}
            Err(e) => {
                last_acp = Err(e);
                break;
            }
        }
    }
    RunKpopAgentLoopsOutcome {
        acp_result: last_acp,
        agent_ran,
    }
}

pub(crate) fn kpop_exp_log_declares_solved(exp_log_path: &PathBuf) -> Result<bool, String> {
    let text = read_exp_log_text(exp_log_path)?;
    Ok(agent_declared_success(&text))
}

pub(crate) fn clear_legacy_gate_exp_log(artifacts: &crate::artifacts::RunArtifacts, max_loops: usize) {
    if max_loops > 1 {
        let _ = std::fs::remove_file(artifacts.gate_exp_log_path(0));
    }
}
#[cfg(test)]
#[path = "kpop_flow_run_loop_test.rs"]
mod kpop_flow_run_loop_test;
#[cfg(test)]
#[path = "kpop_flow_run_loop_kiss_cov_test.rs"]
mod kpop_flow_run_loop_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<KpopLoopExitAfterIteration> = None;
        let _: Option<KpopLoopSnapshot> = None;
        let _: Option<RunKpopAgentLoopsOutcome> = None;
    }
}
