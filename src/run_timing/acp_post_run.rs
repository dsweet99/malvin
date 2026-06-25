//! Shared result merge for ACP runs that emit [`crate::run_timing`] artifacts.

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::artifacts::{SessionDotfileBackups, restore_workspace_session_dotfiles};
use crate::run_timing::RunTiming;

/// Whether an ACP session end should finalize run timing or keep accumulating for a longer workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunTimingSessionEnd {
    /// Standalone command (`kpop`, `do`, …): write final JSON and clear the client timing slot.
    Finalize,
    /// Gate-kpop loop iteration: persist progress JSON; keep wall clock and buckets on the client.
    AccumulateRun,
}

pub fn merge_acp_and_timing_results(
    acp_result: Result<(), String>,
    timing_result: std::io::Result<()>,
) -> Result<(), String> {
    match acp_result {
        Ok(()) => timing_result.map_err(|e| e.to_string()),
        Err(e) => {
            let _ = timing_result;
            Err(e)
        }
    }
}

pub fn prefer_primary_over_secondary(
    primary: Result<(), String>,
    secondary: Result<(), String>,
    both_errors_label: &'static str,
) -> Result<(), String> {
    match (primary, secondary) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) => Err(e),
        (Ok(()), Err(r)) => Err(r),
        (Err(e), Err(r)) => Err(format!("{e}; {both_errors_label}: {r}")),
    }
}

pub fn merge_acp_with_workspace_session_restore(
    primary: Result<(), String>,
    work_dir: &Path,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    let restore_res = restore_workspace_session_dotfiles(work_dir, session_dotfile_backups);
    prefer_primary_over_secondary(primary, restore_res, "workspace session restore failed")
}

pub fn merge_acp_with_custom_restore_and_check_abort(
    primary: Result<(), String>,
    restore_res: Result<(), String>,
    result_path: &Path,
) -> Result<(), String> {
    let merge_result = prefer_primary_over_secondary(primary, restore_res, "workspace session restore failed");
    match crate::orchestrator::check_abort(result_path) {
        Ok(Some(abort)) => match merge_result {
            Ok(()) => Err(format!("ABORT: {abort}")),
            Err(merge_error) => {
                let detail = if merge_error_mentions_restore(&merge_error) {
                    duplicate_safe_restore_error(&merge_error)
                } else {
                    merge_error
                };
                Err(format!("ABORT: {abort}; {detail}"))
            }
        },
        Ok(None) => merge_result,
        Err(e) => Err(format!("cannot read result file for ABORT check: {e}")),
    }
}

pub fn merge_acp_with_workspace_session_restore_and_check_abort(
    primary: Result<(), String>,
    work_dir: &Path,
    session_dotfile_backups: &SessionDotfileBackups,
    result_path: &Path,
) -> Result<(), String> {
    let restore_res = restore_workspace_session_dotfiles(work_dir, session_dotfile_backups);
    merge_acp_with_custom_restore_and_check_abort(primary, restore_res, result_path)
}

pub(crate) fn merge_error_mentions_restore(merge_error: &str) -> bool {
    merge_error.contains("workspace session restore failed:")
        || merge_error.contains("kissconfig restore:")
        || merge_error.contains("malvin_checks restore:")
        || merge_error.contains("kissignore restore:")
        || merge_error.contains("malvin_config restore:")
        || merge_error.contains("gitignore restore:")
        || merge_error.contains("malvin_config_workspace restore:")
}

pub(crate) fn duplicate_safe_restore_error(merge_error: &str) -> String {
    if merge_error_mentions_restore(merge_error) {
        merge_error.to_string()
    } else {
        format!("workspace session restore failed: {merge_error}")
    }
}

pub struct RunTimingAfterAcp<'a> {
    pub client: &'a mut crate::acp::AgentClient,
    pub run_dir: &'a Path,
    pub timing: &'a Arc<Mutex<RunTiming>>,
    pub acp_result: Result<(), String>,
    pub session_end: RunTimingSessionEnd,
}

pub fn emit_run_timing_after_acp(req: RunTimingAfterAcp<'_>) -> Result<(), String> {
    let timing_result = match req.session_end {
        RunTimingSessionEnd::Finalize => {
            crate::run_timing::finalize_run_timing_json_only(req.run_dir, req.timing)
        }
        RunTimingSessionEnd::AccumulateRun => {
            crate::run_timing::persist_open_run_timing_json(req.run_dir, req.timing)
        }
    };
    if matches!(req.session_end, RunTimingSessionEnd::Finalize) {
        req.client.set_run_timing(None);
    }
    merge_acp_and_timing_results(req.acp_result, timing_result)
}

pub fn emit_run_timing_json_only_after_acp(
    client: &mut crate::acp::AgentClient,
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
    acp_result: Result<(), String>,
) -> Result<(), String> {
    emit_run_timing_after_acp(RunTimingAfterAcp {
        client,
        run_dir,
        timing,
        acp_result,
        session_end: RunTimingSessionEnd::Finalize,
    })
}

pub struct RunTimingAfterBackend<'a> {
    pub backend: &'a mut crate::agent_backend::AgentBackend,
    pub run_dir: &'a Path,
    pub timing: &'a Arc<Mutex<RunTiming>>,
    pub agent_result: Result<(), String>,
    pub session_end: RunTimingSessionEnd,
}

pub fn emit_run_timing_after_backend(req: RunTimingAfterBackend<'_>) -> Result<(), String> {
    let timing_result = match req.session_end {
        RunTimingSessionEnd::Finalize => {
            crate::run_timing::finalize_run_timing_json_only(req.run_dir, req.timing)
        }
        RunTimingSessionEnd::AccumulateRun => {
            crate::run_timing::persist_open_run_timing_json(req.run_dir, req.timing)
        }
    };
    if matches!(req.session_end, RunTimingSessionEnd::Finalize) {
        crate::agent_backend::agent_backend_set_run_timing(req.backend, None);
    }
    merge_acp_and_timing_results(req.agent_result, timing_result)
}

pub fn emit_run_timing_json_only_after_backend(
    backend: &mut crate::agent_backend::AgentBackend,
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
    agent_result: Result<(), String>,
) -> Result<(), String> {
    emit_run_timing_after_backend(RunTimingAfterBackend {
        backend,
        run_dir,
        timing,
        agent_result,
        session_end: RunTimingSessionEnd::Finalize,
    })
}

pub fn merge_acp_restore_check_abort_then_print_timing(
    primary: Result<(), String>,
    artifacts: &crate::artifacts::RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    let merged = merge_acp_with_workspace_session_restore_and_check_abort(
        primary,
        &artifacts.work_dir,
        session_dotfile_backups,
        &artifacts.artifact_result_md(),
    );
    crate::run_timing::print_summary_from_run_dir(&artifacts.run_dir).map_err(|e| e.to_string())?;
    merged
}

#[cfg(test)]
mod merge_custom_restore_tests {
    use super::*;

    #[test]
    fn custom_restore_merge_prefers_primary_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = tmp.path().join("result.md");
        std::fs::write(&result, "ok\n").expect("write");
        let err = merge_acp_with_custom_restore_and_check_abort(
            Err("agent failed".to_string()),
            Ok(()),
            &result,
        )
        .expect_err("primary");
        assert!(err.contains("agent failed"));
    }
}
