
use std::path::Path;

use crate::artifacts::SessionDotfileBackups;
use crate::cli::workflow_kpop_shared::run_kpop_workspace_gates;
use crate::kpop_progression::{agent_declared_success, read_exp_log_text};

use super::behavior::KPopHardConstraints;

pub(crate) struct GateLoopExitCtx<'a> {
    pub behavior: KPopHardConstraints,
    pub artifacts: &'a crate::artifacts::RunArtifacts,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
}

pub(crate) fn run_gate_workspace_gates_with_fresh_backups(
    artifacts: &crate::artifacts::RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
    behavior: KPopHardConstraints,
) -> bool {
    run_kpop_workspace_gates(
        artifacts,
        session_dotfile_backups,
        behavior.restore_malvin_checks_after_session(),
    )
    .is_ok()
}

pub(crate) fn session_wrote_kpop_solved(exp_log_path: &Path) -> Result<bool, String> {
    let text = read_exp_log_text(exp_log_path)?;
    Ok(agent_declared_success(&text))
}

pub(crate) fn kpop_solved_early_exit(
    behavior: KPopHardConstraints,
    consecutive_solved: usize,
    artifacts: &crate::artifacts::RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
) -> bool {
    if consecutive_solved < behavior.consecutive_kpop_solved_to_exit() {
        return false;
    }
    if behavior.require_passing_gates_for_exit() && !behavior.skip_workspace_quality_gates {
        run_gate_workspace_gates_with_fresh_backups(artifacts, session_dotfile_backups, behavior)
    } else {
        true
    }
}

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

pub(crate) fn gate_loop_early_exit(ctx: &GateLoopExitCtx<'_>) -> bool {
    if ctx.behavior.skip_workspace_quality_gates {
        return false;
    }
    ctx.behavior.require_passing_gates_for_exit()
        && run_gate_workspace_gates_with_fresh_backups(
            ctx.artifacts,
            ctx.session_dotfile_backups,
            ctx.behavior,
        )
}
