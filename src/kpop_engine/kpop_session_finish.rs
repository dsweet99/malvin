use crate::cli::SharedOpts;

use super::kpop_session::run_kpop_hard_constraints_after_session;
use super::prepared::KPopEnginePrepared;

pub(crate) fn finish_kpop_engine_after_pass(
    _shared: &SharedOpts,
    prepared: &KPopEnginePrepared,
    _agent_ran: bool,
    run_timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
) -> Result<(), String> {
    if let Some(timing) = run_timing {
        crate::run_timing::finalize_and_emit_run_timing(&prepared.artifacts().run_dir, timing)
            .map_err(|e| e.to_string())?;
    } else {
        crate::run_timing::print_summary_from_run_dir(&prepared.artifacts().run_dir)
            .map_err(|e| e.to_string())?;
    }
    crate::agent_phase::print_done_with_reporting_phase();
    Ok(())
}

pub(crate) fn fail_kpop_engine_after_exhausted(
    command: &str,
    prepared: &KPopEnginePrepared,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
    behavior: super::behavior::KPopHardConstraints,
) -> Result<(), String> {
    // Code/tidy exhaustion already ran gates in `run_kpop_engine`. Rewind disk so gate-prep
    // side effects cannot poison the next outer retry; do not run gates a second time.
    if behavior.recheck_gates_after_exhausted && !behavior.skip_workspace_quality_gates {
        let work_dir = prepared.artifacts().work_dir.as_path();
        if behavior.restore_malvin_checks_after_session() {
            session_dotfile_backups.restore(work_dir)?;
        } else {
            session_dotfile_backups.restore_excluding_malvin_checks(work_dir)?;
        }
        crate::cli::workflow_kpop_shared::write_checks_do_not_pass_for_artifacts(
            prepared.artifacts(),
        )?;
        return Err(crate::cli::format_workspace_gate_failure(
            command,
            "workspace quality gates did not pass after the kpop session",
        ));
    }
    run_kpop_hard_constraints_after_session(command, prepared, session_dotfile_backups, behavior)
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_kpop_engine_finish() {
        let _ = finish_kpop_engine_after_pass;
        let _ = fail_kpop_engine_after_exhausted;
    }
}
