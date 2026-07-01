//! Gate-loop exit predicates for MPC done (see `concepts_2.md` §5).

use std::path::Path;

use crate::artifacts::SessionDotfileBackups;
use crate::mpc_planning_brief::MpcPlanningBriefAspect;
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

pub(crate) fn mpc_done_early_exit(
    ctx: &GateLoopExitCtx<'_>,
    brief_path: &Path,
) -> Result<bool, String> {
    let _aspect = MpcPlanningBriefAspect::ExitGateIntegration;
    if !super::mpc_planner::user_brief_declares_mpc_done(brief_path)? {
        let _done_aspect = MpcPlanningBriefAspect::DoneMarkerDetection;
        return Ok(false);
    }
    Ok(gates_pass_for_exit(ctx))
}

fn gates_pass_for_exit(ctx: &GateLoopExitCtx<'_>) -> bool {
    if ctx.behavior.require_passing_gates_for_exit() && !ctx.behavior.skip_workspace_quality_gates {
        run_gate_workspace_gates_with_fresh_backups(
            ctx.artifacts,
            ctx.session_dotfile_backups,
            ctx.behavior,
        )
    } else {
        true
    }
}
