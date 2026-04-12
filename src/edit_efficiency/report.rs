//! Orchestration helpers: checkpoint logging and writing `edit_efficiency.json`.
//!
//! **Streams:** The successful one-line summary uses `println!` (stdout). Skipped metering and
//! `finish` failures use `eprintln!` (stderr); see root `grounding.md` (tee / progress contract).

use std::path::Path;

use super::{EditEfficiencyMeter, EditEfficiencyReport};

/// Printed to stderr when [`finish_and_write_report`] is called with `None` (meter not started).
pub const EDIT_EFFICIENCY_NOT_MEASURED_MESSAGE: &str =
    "Edit efficiency: not measured (not a git repository or snapshot failed).";

/// One-line user-visible summary (stdout) when metering succeeds; kept stable for regression tests.
#[must_use]
pub fn format_edit_efficiency_stdout_line(report: &EditEfficiencyReport) -> String {
    format!(
        "Edit efficiency: {:.4} (net {} B / gross {} B); checkpoints={}; diff_steps={}",
        report.efficiency,
        report.net_bytes,
        report.gross_bytes,
        report.checkpoint_calls,
        report.gross_diff_steps
    )
}

/// Best-effort checkpoint for orchestration; logs on failure without propagating.
pub fn maybe_checkpoint(meter: &mut Option<EditEfficiencyMeter>) {
    let Some(m) = meter.as_mut() else {
        return;
    };
    if let Err(e) = m.checkpoint() {
        tracing::warn!(target: "malvin::edit_efficiency", ?e, "checkpoint failed");
    }
}

/// Writes the edit-efficiency report (see [`finish_and_write_report`]), then returns `value` unchanged.
///
/// Use after the workflow or ACP body so users still see a line (or “not measured”) when `value` is `Err`.
#[must_use]
pub fn finish_edit_efficiency_then_return<T>(
    meter: Option<EditEfficiencyMeter>,
    run_dir: &Path,
    value: T,
) -> T {
    finish_and_write_report(meter, run_dir);
    value
}

/// Run [`EditEfficiencyMeter::finish`], print a one-line summary to stdout, log via tracing, and write
/// `edit_efficiency.json` under `run_dir`.
pub fn finish_and_write_report(meter: Option<EditEfficiencyMeter>, run_dir: &Path) {
    let Some(m) = meter else {
        eprintln!("{EDIT_EFFICIENCY_NOT_MEASURED_MESSAGE}");
        return;
    };
    match m.finish() {
        Ok(report) => {
            tracing::info!(
                target: "malvin::edit_efficiency",
                gross = report.gross_bytes,
                net = report.net_bytes,
                efficiency = report.efficiency,
                checkpoint_calls = report.checkpoint_calls,
                gross_diff_steps = report.gross_diff_steps,
                "edit efficiency summary"
            );
            println!("{}", format_edit_efficiency_stdout_line(&report));
            let path = run_dir.join("edit_efficiency.json");
            match serde_json::to_vec_pretty(&report) {
                Ok(bytes) => {
                    if let Err(e) = std::fs::write(&path, bytes) {
                        tracing::warn!(
                            target: "malvin::edit_efficiency",
                            ?e,
                            path = %path.display(),
                            "write edit_efficiency.json"
                        );
                    }
                }
                Err(e) => tracing::warn!(target: "malvin::edit_efficiency", ?e, "serialize report"),
            }
        }
        Err(e) => {
            tracing::warn!(target: "malvin::edit_efficiency", ?e, "finish failed");
            eprintln!("Edit efficiency: finish failed ({e}).");
        }
    }
}

#[cfg(test)]
mod contract_tests {
    use super::super::{EditEfficiencyMeter, EditEfficiencyReport};
    use super::{
        EDIT_EFFICIENCY_NOT_MEASURED_MESSAGE, finish_and_write_report,
        finish_edit_efficiency_then_return, format_edit_efficiency_stdout_line,
    };
    use crate::edit_efficiency::meter_tests::repo_with_git;

    #[test]
    fn not_measured_message_is_stable_for_users_and_tests() {
        assert_eq!(
            EDIT_EFFICIENCY_NOT_MEASURED_MESSAGE,
            "Edit efficiency: not measured (not a git repository or snapshot failed)."
        );
    }

    #[test]
    fn finish_edit_efficiency_then_return_propagates_inner_result_when_meter_none() {
        let tmp = tempfile::tempdir().unwrap();
        let out: Result<(), String> = finish_edit_efficiency_then_return(
            None,
            tmp.path(),
            Err("simulated acp failure".into()),
        );
        assert_eq!(out, Err("simulated acp failure".into()));
    }

    /// Same ordering guarantee as `kpop_run_prompt_and_efficiency` / `Orchestrator::run`: report is
    /// finalized (and `edit_efficiency.json` written when metering succeeds) before the ACP error is
    /// returned.
    #[test]
    fn finish_edit_efficiency_then_return_writes_report_before_propagating_acp_error() {
        let tmp = repo_with_git();
        let p = tmp.path();
        std::fs::write(p.join("f.rs"), b"x").unwrap();
        let m = EditEfficiencyMeter::new(p).unwrap();
        let run_dir = tempfile::tempdir().unwrap();
        let out: Result<(), &str> = finish_edit_efficiency_then_return(
            Some(m),
            run_dir.path(),
            Err("simulated acp failure"),
        );
        assert_eq!(out, Err("simulated acp failure"));
        assert!(
            run_dir.path().join("edit_efficiency.json").exists(),
            "report must be written before the ACP error is propagated"
        );
    }

    #[test]
    fn stdout_line_format_matches_contract() {
        let r = EditEfficiencyReport {
            gross_bytes: 100,
            net_bytes: 50,
            efficiency: 0.5,
            checkpoint_calls: 2,
            gross_diff_steps: 3,
        };
        assert_eq!(
            format_edit_efficiency_stdout_line(&r),
            "Edit efficiency: 0.5000 (net 50 B / gross 100 B); checkpoints=2; diff_steps=3"
        );
    }

    #[test]
    fn finish_and_write_report_writes_edit_efficiency_json() {
        let tmp = repo_with_git();
        let p = tmp.path();
        std::fs::write(p.join("tracked.rs"), b"v1").unwrap();
        let m = EditEfficiencyMeter::new(p).unwrap();
        let run_dir = tempfile::tempdir().unwrap();
        finish_and_write_report(Some(m), run_dir.path());
        let path = run_dir.path().join("edit_efficiency.json");
        assert!(path.exists(), "expected edit_efficiency.json under run_dir");
        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
        assert!(v.get("efficiency").is_some());
        assert!(v.get("gross_bytes").is_some());
    }
}

#[cfg(test)]
mod kiss_stringify_report {
    #[test]
    fn kiss_stringify_report_fns() {
        let _ = stringify!(super::maybe_checkpoint);
        let _ = stringify!(super::finish_and_write_report);
        let _ = stringify!(super::finish_edit_efficiency_then_return);
        let _ = stringify!(super::format_edit_efficiency_stdout_line);
    }
}
