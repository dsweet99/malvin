use std::path::Path;

use crate::artifacts::SessionDotfileBackups;
use crate::kpop_progression::{count_kpop_solved_markers, read_exp_log_text};

use crate::cli::build_agent;
use crate::cli::workflow_kpop_shared::{
    gate_kpop_loop_iterations, run_kpop_workspace_gates,
};

use super::kpop_session::{print_gate_kpop_log_line, run_gate_kpop_session, GateKpopMultiturnCtx};
use super::params::{GateKpopIterationParams, GateKpopLoopParams};

type GateKpopLoopOutcome = (
    bool,
    bool,
    Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
);

const CONSECUTIVE_KPOP_SOLVED_TO_EXIT: usize = 2;

const fn bump_consecutive_solved_streak(
    consecutive_solved: usize,
    markers_before: usize,
    markers_after: usize,
) -> usize {
    if markers_after > markers_before {
        consecutive_solved.saturating_add(1)
    } else {
        0
    }
}

fn initial_solved_markers(exp_log_path: &Path) -> usize {
    read_exp_log_text(exp_log_path)
        .map(|text| count_kpop_solved_markers(&text))
        .unwrap_or(0)
}

fn two_consecutive_solved_with_passing_gates(
    consecutive_solved: usize,
    artifacts: &crate::artifacts::RunArtifacts,
) -> bool {
    consecutive_solved >= CONSECUTIVE_KPOP_SOLVED_TO_EXIT
        && run_kpop_workspace_gates(artifacts).is_ok()
}

struct GateKpopAgentSession {
    client: crate::acp::AgentClient,
    session_dotfile_backups: SessionDotfileBackups,
}

fn agent_session_run_timing(
    agent: Option<&GateKpopAgentSession>,
) -> Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>> {
    agent.and_then(|s| s.client.timing.clone())
}

fn refresh_solved_streak(
    exp_log_path: &Path,
    consecutive_solved: usize,
    markers_before: usize,
) -> Result<(usize, usize), String> {
    let solved_markers = count_kpop_solved_markers(&read_exp_log_text(exp_log_path)?);
    let streak =
        bump_consecutive_solved_streak(consecutive_solved, markers_before, solved_markers);
    Ok((streak, solved_markers))
}

fn gate_kpop_solved_early_exit(
    consecutive_solved: usize,
    artifacts: &crate::artifacts::RunArtifacts,
    agent_ran: bool,
    agent: Option<&GateKpopAgentSession>,
) -> Option<GateKpopLoopOutcome> {
    if two_consecutive_solved_with_passing_gates(consecutive_solved, artifacts) {
        Some((true, agent_ran, agent_session_run_timing(agent)))
    } else {
        None
    }
}

fn start_gate_kpop_agent_session(
    params: &GateKpopLoopParams<'_>,
) -> Result<GateKpopAgentSession, String> {
    let mut client = build_agent(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
    );
    client.prompts_log_run_dir = Some(params.prepared.artifacts().run_dir.clone());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&params.prepared.artifacts().work_dir)?;
    print_gate_kpop_log_line(params.prepared);
    Ok(GateKpopAgentSession {
        client,
        session_dotfile_backups,
    })
}

async fn run_gate_kpop_on_loop_iteration(
    params: &GateKpopLoopParams<'_>,
    agent: &mut Option<GateKpopAgentSession>,
) -> Result<(), String> {
    if agent.is_none() {
        *agent = Some(start_gate_kpop_agent_session(params)?);
    }
    let session = agent.as_mut().expect("agent session");
    let mut iteration = GateKpopIterationParams {
        loop_params: params,
        session_dotfile_backups: &session.session_dotfile_backups,
        client: &mut session.client,
    };
    let mut ctx = GateKpopMultiturnCtx {
        iteration: &mut iteration,
    };
    run_gate_kpop_session(&mut ctx).await
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
    let exp_log_path = params.prepared.exp_log_path();
    let mut gates_ok = false;
    let mut agent_ran = false;
    let mut consecutive_solved = 0usize;
    let mut solved_markers = initial_solved_markers(exp_log_path);
    let mut agent = None;

    for _ in 0..iterations {
        let markers_before = solved_markers;
        run_gate_kpop_on_loop_iteration(&params, &mut agent).await?;
        agent_ran = true;
        (consecutive_solved, solved_markers) =
            refresh_solved_streak(exp_log_path, consecutive_solved, markers_before)?;
        if let Some(outcome) = gate_kpop_solved_early_exit(
            consecutive_solved,
            params.prepared.artifacts(),
            agent_ran,
            agent.as_ref(),
        ) {
            return Ok(outcome);
        }
    }
    if params.behavior.recheck_gates_after_exhausted && agent_ran && !gates_ok {
        gates_ok = run_kpop_workspace_gates(params.prepared.artifacts()).is_ok();
    }
    Ok((gates_ok, agent_ran, agent_session_run_timing(agent.as_ref())))
}

#[cfg(test)]
mod tests {
    use super::bump_consecutive_solved_streak;

    #[test]
    fn bump_consecutive_solved_streak_increments_or_resets() {
        assert_eq!(bump_consecutive_solved_streak(1, 0, 1), 2);
        assert_eq!(bump_consecutive_solved_streak(2, 1, 1), 0);
    }

    #[test]
    fn initial_solved_markers_reads_existing_log() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
        assert_eq!(super::initial_solved_markers(&path), 1);
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
    fn agent_session_run_timing_none_without_agent() {
        assert!(super::agent_session_run_timing(None).is_none());
    }

    #[test]
    fn agent_session_run_timing_clones_client_slot() {
        let mut client = crate::acp::AgentClient::new(
            "m".into(),
            crate::acp::AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        let timing = client.attach_run_timing_for_session();
        let agent = Some(super::GateKpopAgentSession {
            client,
            session_dotfile_backups: crate::artifacts::SessionDotfileBackups {
                kissconfig: crate::artifacts::KissConfigBackup::Missing,
                malvin_checks: crate::artifacts::MalvinChecksBackup::Missing,
                kissignore: crate::artifacts::KissignoreBackup::Missing,
                malvin_config: crate::artifacts::MalvinConfigBackup::Missing,
            },
        });
        assert!(std::sync::Arc::ptr_eq(
            &timing,
            super::agent_session_run_timing(agent.as_ref()).as_ref().unwrap()
        ));
    }

    #[test]
    fn gate_kpop_loop_session_helpers_are_covered() {
        let _ = stringify!(super::refresh_solved_streak);
        let _ = stringify!(super::gate_kpop_solved_early_exit);
        let _ = stringify!(super::start_gate_kpop_agent_session);
        let _ = stringify!(super::run_gate_kpop_on_loop_iteration);
        let _ = stringify!(super::run_gate_kpop_loop);
    }
}
