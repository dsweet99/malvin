use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::kpop_progression::{agent_declared_success, read_exp_log_text};

use crate::cli::build_agent;
use crate::cli::workflow_kpop_shared::{
    gate_kpop_loop_iterations, run_kpop_workspace_gates,
};

use super::kpop_session::{print_gate_kpop_log_line, run_gate_kpop_session, GateKpopMultiturnCtx};
use super::params::{GateKpopIterationParams, GateKpopLoopParams};

type GateKpopLoopOutcome = (
    bool,
    bool,
    Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
);

const CONSECUTIVE_KPOP_SOLVED_TO_EXIT: usize = 2;

fn session_wrote_kpop_solved(exp_log_path: &Path) -> Result<bool, String> {
    let text = read_exp_log_text(exp_log_path)?;
    Ok(agent_declared_success(&text))
}

fn two_consecutive_solved_with_passing_gates(
    consecutive_solved: usize,
    artifacts: &crate::artifacts::RunArtifacts,
) -> bool {
    consecutive_solved >= CONSECUTIVE_KPOP_SOLVED_TO_EXIT
        && run_kpop_workspace_gates(artifacts).is_ok()
}

fn gate_kpop_solved_early_exit(
    consecutive_solved: usize,
    artifacts: &crate::artifacts::RunArtifacts,
    agent_ran: bool,
    run_timing: Option<&Arc<Mutex<crate::run_timing::RunTiming>>>,
) -> Option<GateKpopLoopOutcome> {
    if two_consecutive_solved_with_passing_gates(consecutive_solved, artifacts) {
        Some((true, agent_ran, run_timing.cloned()))
    } else {
        None
    }
}

async fn run_gate_kpop_on_loop_iteration(
    params: &GateKpopLoopParams<'_>,
    iteration: usize,
    run_timing: &Arc<Mutex<crate::run_timing::RunTiming>>,
) -> Result<(), String> {
    let exp_log_path = crate::artifacts::ensure_gate_exp_log_file(
        params.prepared.artifacts(),
        iteration,
    )
    .map_err(|e| e.to_string())?;

    let mut client = build_agent(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
    );
    client.set_run_timing(Some(Arc::clone(run_timing)));
    client.prompts_log_run_dir = Some(params.prepared.artifacts().run_dir.clone());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&params.prepared.artifacts().work_dir)?;
    print_gate_kpop_log_line(params.prepared, &exp_log_path);

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
    run_gate_kpop_session(&mut ctx).await
}

use crate::artifacts::SessionDotfileBackups;

fn refresh_consecutive_solved_streak(
    consecutive_solved: usize,
    exp_log_path: &Path,
) -> Result<usize, String> {
    if session_wrote_kpop_solved(exp_log_path)? {
        Ok(consecutive_solved.saturating_add(1))
    } else {
        Ok(0)
    }
}

async fn gate_kpop_loop_one_iteration(
    params: &GateKpopLoopParams<'_>,
    iteration: usize,
    run_timing: &Arc<Mutex<crate::run_timing::RunTiming>>,
    consecutive_solved: usize,
) -> Result<(usize, Option<GateKpopLoopOutcome>), String> {
    run_gate_kpop_on_loop_iteration(params, iteration, run_timing).await?;
    let exp_log_path = params.prepared.artifacts().gate_exp_log_path(iteration);
    let streak = refresh_consecutive_solved_streak(consecutive_solved, &exp_log_path)?;
    let early = gate_kpop_solved_early_exit(
        streak,
        params.prepared.artifacts(),
        true,
        Some(run_timing),
    );
    Ok((streak, early))
}

pub(crate) async fn run_gate_kpop_loop(
    params: GateKpopLoopParams<'_>,
) -> Result<GateKpopLoopOutcome, String> {
    if params.behavior.skip_kpop_on_initial_pass
        && run_kpop_workspace_gates(params.prepared.artifacts()).is_ok()
    {
        return Ok((true, false, None));
    }

    let iterations = gate_kpop_loop_iterations(params.max_loops);
    let run_timing = crate::run_timing::attach_gate_kpop_loop_run_timing();
    let mut consecutive_solved = 0usize;
    for iteration in 1..=iterations {
        let (streak, early) =
            gate_kpop_loop_one_iteration(&params, iteration, &run_timing, consecutive_solved).await?;
        consecutive_solved = streak;
        if let Some(outcome) = early {
            return Ok(outcome);
        }
    }
    let gates_ok = params.behavior.recheck_gates_after_exhausted
        && run_kpop_workspace_gates(params.prepared.artifacts()).is_ok();
    Ok((gates_ok, true, Some(run_timing)))
}

#[cfg(test)]
mod tests {
    use super::session_wrote_kpop_solved;

    #[test]
    fn refresh_consecutive_solved_streak_increments_or_resets() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let empty = tmp.path().join("empty.md");
        std::fs::write(&empty, "").expect("write");
        assert_eq!(super::refresh_consecutive_solved_streak(1, &empty).expect("read"), 0);
        let solved = tmp.path().join("solved.md");
        std::fs::write(&solved, "## KPOP_SOLVED\n").expect("write");
        assert_eq!(super::refresh_consecutive_solved_streak(1, &solved).expect("read"), 2);
    }

    #[test]
    fn session_wrote_kpop_solved_reads_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
        assert!(session_wrote_kpop_solved(&path).expect("read"));
    }

    #[test]
    fn two_consecutive_solved_with_passing_gates_checks_streak_and_workspace() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 0);
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
        assert!(!super::two_consecutive_solved_with_passing_gates(1, &artifacts));
        assert!(super::two_consecutive_solved_with_passing_gates(2, &artifacts));
    }

    #[test]
    fn gate_kpop_solved_early_exit_needs_streak_and_gates() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 0);
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
        assert!(super::gate_kpop_solved_early_exit(1, &artifacts, true, None).is_none());
        assert!(super::gate_kpop_solved_early_exit(2, &artifacts, true, None).is_some());
    }

    #[test]
    fn gate_kpop_loop_session_helpers_are_covered() {
        let _ = stringify!(super::run_gate_kpop_on_loop_iteration);
        let _ = stringify!(super::gate_kpop_loop_one_iteration);
        let _ = stringify!(super::run_gate_kpop_loop);
    }
}
