//! Shared result merge for ACP runs that emit [`crate::run_timing`] artifacts.

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::artifacts::{SessionDotfileBackups, restore_workspace_session_dotfiles};
use crate::run_timing::RunTiming;

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

pub fn merge_acp_with_workspace_session_restore_and_check_abort(
    primary: Result<(), String>,
    work_dir: &Path,
    session_dotfile_backups: &SessionDotfileBackups,
    result_path: &Path,
) -> Result<(), String> {
    let merge_result =
        merge_acp_with_workspace_session_restore(primary, work_dir, session_dotfile_backups);
    if let Some(abort) = crate::orchestrator::check_abort(result_path) {
        return match merge_result {
            Ok(()) => Err(format!("ABORT: {abort}")),
            Err(merge_error) => {
                let detail = if merge_error.contains("workspace session restore failed:") {
                    duplicate_safe_restore_error(&merge_error)
                } else {
                    merge_error
                };
                Err(format!("ABORT: {abort}; {detail}"))
            }
        };
    }
    merge_result
}

pub(crate) fn merge_error_mentions_restore(merge_error: &str) -> bool {
    merge_error.contains("workspace session restore failed:")
        || merge_error.contains("kissconfig restore:")
        || merge_error.contains("malvin_checks restore:")
        || merge_error.contains("kissignore restore:")
}

pub(crate) fn duplicate_safe_restore_error(merge_error: &str) -> String {
    if merge_error_mentions_restore(merge_error) {
        merge_error.to_string()
    } else {
        format!("workspace session restore failed: {merge_error}")
    }
}

pub fn emit_run_timing_after_acp(
    client: &mut crate::acp::AgentClient,
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
    acp_result: Result<(), String>,
) -> Result<(), String> {
    let timing_result = crate::run_timing::finalize_and_emit_run_timing(run_dir, timing);
    client.set_run_timing(None);
    merge_acp_and_timing_results(acp_result, timing_result)
}

pub fn emit_run_timing_json_only_after_acp(
    client: &mut crate::acp::AgentClient,
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
    acp_result: Result<(), String>,
) -> Result<(), String> {
    let timing_result = crate::run_timing::finalize_run_timing_json_only(run_dir, timing);
    client.set_run_timing(None);
    merge_acp_and_timing_results(acp_result, timing_result)
}
