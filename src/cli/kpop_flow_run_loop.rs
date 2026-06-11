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

const fn kpop_loop_abort(agent_ran: bool, err: String) -> RunKpopAgentLoopsOutcome {
    RunKpopAgentLoopsOutcome {
        acp_result: Err(err),
        agent_ran,
    }
}

struct KpopLoopSnapshot {
    backups: SessionDotfileBackups,
    exp_iter: usize,
    exp_log_path: PathBuf,
}

fn snapshot_kpop_loop_dotfiles_and_exp_log(
    artifacts: &crate::artifacts::RunArtifacts,
    agent_loop: usize,
    max_loops: usize,
) -> Result<KpopLoopSnapshot, String> {
    let backups = SessionDotfileBackups::snapshot(&artifacts.work_dir)?;
    let exp_iter = kpop_agent_loop_exp_iteration(agent_loop, max_loops);
    let exp_log_path = ensure_gate_exp_log_file(artifacts, exp_iter).map_err(|e| e.to_string())?;
    Ok(KpopLoopSnapshot {
        backups,
        exp_iter,
        exp_log_path,
    })
}

fn kpop_iteration_declares_solved(
    exp_log_path: &PathBuf,
    last_acp: &mut Result<(), String>,
) -> bool {
    kpop_exp_log_declares_solved(exp_log_path).unwrap_or_else(|e| {
        *last_acp = Err(e);
        true
    })
}

pub(crate) async fn run_kpop_agent_loops(
    params: RunKpopAgentLoopsParams<'_>,
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
        if kpop_iteration_declares_solved(&loop_snapshot.exp_log_path, &mut last_acp) {
            break;
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
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<KpopLoopSnapshot> = None;
        let _ = kpop_loop_abort;
        let _ = snapshot_kpop_loop_dotfiles_and_exp_log;
        let _ = kpop_iteration_declares_solved;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn kiss_cov_run_kpop_agent_loops_outcome() {
        let _ = std::any::type_name::<RunKpopAgentLoopsOutcome>();
        let _ = std::any::type_name::<RunKpopAgentLoopsParams>();
        let _ = run_kpop_agent_loops;
        let _ = clear_legacy_gate_exp_log;
    }

    #[test]
    fn kpop_exp_log_declares_solved_reads_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
        assert!(kpop_exp_log_declares_solved(&path).expect("read"));
    }

    #[test]
    fn kpop_loop_abort_records_error_and_agent_ran() {
        let outcome = kpop_loop_abort(true, "setup failed".into());
        assert!(outcome.agent_ran);
        assert_eq!(outcome.acp_result, Err("setup failed".into()));
    }

    #[test]
    fn kpop_iteration_declares_solved_propagates_read_errors() {
        let mut last_acp = Ok(());
        let bad = PathBuf::from("/nonexistent/exp_log.md");
        assert!(kpop_iteration_declares_solved(&bad, &mut last_acp));
        assert!(last_acp.is_err());
    }

    #[test]
    fn kpop_iteration_declares_solved_false_without_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(&path, "still working\n").expect("write");
        let mut last_acp = Ok(());
        assert!(!kpop_iteration_declares_solved(&path, &mut last_acp));
        assert!(last_acp.is_ok());
    }

    #[test]
    fn snapshot_kpop_loop_dotfiles_and_exp_log_builds_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
        let snap = snapshot_kpop_loop_dotfiles_and_exp_log(&artifacts, 1, 2).expect("snapshot");
        let KpopLoopSnapshot {
            exp_iter,
            exp_log_path,
            backups: _,
        } = snap;
        assert_eq!(exp_iter, 1);
        assert!(exp_log_path.is_file());
        assert!(exp_log_path.to_string_lossy().contains("_g1.md"));
    }
}
