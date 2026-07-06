//! Gate-loop exit predicates.

use crate::artifacts::SessionDotfileBackups;
use crate::cli::workflow_kpop_shared::run_kpop_workspace_gates;

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
