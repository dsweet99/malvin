//! Shared result merge for ACP runs that emit [`crate::run_timing`] artifacts.

use std::path::Path;
use std::sync::{Arc, Mutex};

use malvin::acp::AgentClient;
use malvin::artifacts::{GroundingBackup, restore_workspace_grounding};
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

pub fn merge_acp_with_grounding_restore(
    primary: Result<(), String>,
    work_dir: &Path,
    grounding_backup: &GroundingBackup,
) -> Result<(), String> {
    let restore_res = restore_workspace_grounding(work_dir, grounding_backup);
    prefer_primary_over_secondary(primary, restore_res, "grounding restore failed")
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
    use super::{merge_acp_and_timing_results, prefer_primary_over_secondary};

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
                "grounding restore failed",
            ),
            Err("wf; grounding restore failed: restore".into())
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
    }

    #[test]
    fn prefer_primary_surfaces_primary_when_secondary_ok() {
        assert_eq!(
            prefer_primary_over_secondary(Err("wf".into()), Ok(()), "x"),
            Err("wf".into())
        );
    }
}
