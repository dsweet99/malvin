//! Shared result merge for ACP runs that emit [`crate::run_timing`] artifacts.

use std::path::Path;
use std::sync::{Arc, Mutex};

use malvin::acp::AgentClient;
use malvin::artifacts::{
    KissConfigBackup, SessionDotfileBackups, restore_workspace_kissconfig_backup,
    restore_workspace_session_dotfiles,
};
use malvin::run_timing::RunTiming;

/// Prefer ACP failures over run-timing artifact errors once run timing emission completes.
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

pub fn merge_acp_with_kissconfig_restore(
    primary: Result<(), String>,
    work_dir: &Path,
    kissconfig_backup: &KissConfigBackup,
) -> Result<(), String> {
    let restore_res = restore_workspace_kissconfig_backup(work_dir, kissconfig_backup);
    prefer_primary_over_secondary(primary, restore_res, "kissconfig restore failed")
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
    if let Some(abort) = abort_message_from_result_md(result_path) {
        return match merge_result {
            Ok(()) => Err(format!("ABORT: {abort}")),
            Err(merge_error) => Err(format!(
                "ABORT: {abort}; {}",
                duplicate_safe_restore_error(&merge_error)
            )),
        };
    }
    merge_result
}

fn duplicate_safe_restore_error(merge_error: &str) -> String {
    if merge_error.contains("workspace session restore failed:")
        || merge_error.contains("kissconfig restore failed:")
        || merge_error.contains("malvin_checks restore failed:")
        || merge_error.contains("kissignore restore failed:")
    {
        merge_error.to_string()
    } else {
        format!("workspace session restore failed: {merge_error}")
    }
}

fn abort_message_from_result_md(result_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(result_path).ok()?;
    let text = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("ABORT:") {
            return Some(rest.trim_start().to_string());
        }
    }
    None
}

/// After ACP work: write `run_timing.json`, print the stdout timing summary line (starts with [`malvin::run_timing::RUN_TIMING_SUMMARY_PREFIX`], i.e. `TIMING: ` with one ASCII space after the colon before the first field), clear [`AgentClient`] timing slot, merge errors.
pub fn emit_run_timing_after_acp(
    client: &mut AgentClient,
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
    acp_result: Result<(), String>,
) -> Result<(), String> {
    let timing_result = malvin::run_timing::finalize_and_emit_run_timing(run_dir, timing);
    client.set_run_timing(None);
    merge_acp_and_timing_results(acp_result, timing_result)
}

/// After ACP work: write `run_timing.json` without a stdout timing line, clear [`AgentClient`]
/// timing slot, merge errors.
pub fn emit_run_timing_json_only_after_acp(
    client: &mut AgentClient,
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
    acp_result: Result<(), String>,
) -> Result<(), String> {
    let timing_result = malvin::run_timing::finalize_run_timing_json_only(run_dir, timing);
    client.set_run_timing(None);
    merge_acp_and_timing_results(acp_result, timing_result)
}

#[cfg(test)]
mod tests {
    use super::{
        duplicate_safe_restore_error, merge_acp_and_timing_results, prefer_primary_over_secondary,
    };

    #[test]
    fn merge_timing_ok_acp_ok_propagates_timing_err() {
        assert_eq!(
            merge_acp_and_timing_results(Ok(()), Err(std::io::Error::other("disk"))),
            Err("disk".to_string())
        );
    }

    #[test]
    fn merge_timing_ok_acp_err_drops_timing_result() {
        assert_eq!(
            merge_acp_and_timing_results(Err("acp".into()), Err(std::io::Error::other("disk"))),
            Err("acp".into())
        );
    }

    #[test]
    fn merge_both_ok() {
        assert_eq!(merge_acp_and_timing_results(Ok(()), Ok(())), Ok(()));
    }

    #[test]
    fn prefer_primary_appends_secondary_error_when_primary_fails() {
        assert_eq!(
            prefer_primary_over_secondary(
                Err("wf".into()),
                Err("restore".into()),
                "workspace session restore failed",
            ),
            Err("wf; workspace session restore failed: restore".into())
        );
    }

    #[test]
    fn prefer_primary_surfaces_secondary_when_primary_ok() {
        assert_eq!(
            prefer_primary_over_secondary(Ok(()), Err("restore".into()), "x"),
            Err("restore".into())
        );
    }

    #[test]
    fn prefer_primary_ok_when_both_ok() {
        assert_eq!(prefer_primary_over_secondary(Ok(()), Ok(()), "x"), Ok(()));
    }

    #[test]
    fn kiss_stringify_timing_merge_units() {
        let _ = stringify!(crate::cli::timing_merge::emit_run_timing_json_only_after_acp);
        let _ = stringify!(crate::cli::timing_merge::merge_acp_with_kissconfig_restore);
    }

    #[test]
    fn prefer_primary_surfaces_primary_when_secondary_ok() {
        assert_eq!(
            prefer_primary_over_secondary(Err("wf".into()), Ok(()), "x"),
            Err("wf".into())
        );
    }

    #[test]
    fn duplicate_safe_restore_error_does_not_repeat_restore_prefix() {
        assert_eq!(
            duplicate_safe_restore_error("wf failed; workspace session restore failed: restore")
                .as_str(),
            "wf failed; workspace session restore failed: restore"
        );
    }

    #[test]
    fn duplicate_safe_restore_error_adds_restore_prefix_when_missing() {
        assert_eq!(
            duplicate_safe_restore_error("wf failed"),
            "workspace session restore failed: wf failed"
        );
    }
}
