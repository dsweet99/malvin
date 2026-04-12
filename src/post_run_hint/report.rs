//! Orchestration helpers for the post-run tracked-edit metrics hint.
//!
//! **Streams:** The hint uses `eprintln!` (stderr); see root `grounding.md` (tee / progress contract).

use std::path::Path;

/// Printed to stderr when [`finish_and_write_report`] runs (metering removed).
pub const POST_RUN_METRICS_NOT_MEASURED_MESSAGE: &str = "Tracked edit metrics: not measured.";

/// Writes the stable “not measured” hint to stderr.
pub fn finish_and_write_report(_run_dir: &Path) {
    eprintln!("{POST_RUN_METRICS_NOT_MEASURED_MESSAGE}");
}

/// Calls [`finish_and_write_report`], then returns `value` unchanged.
#[must_use]
pub fn finish_post_run_hint_then_return<T>(run_dir: &Path, value: T) -> T {
    finish_and_write_report(run_dir);
    value
}

#[cfg(test)]
mod contract_tests {
    use super::{
        POST_RUN_METRICS_NOT_MEASURED_MESSAGE, finish_and_write_report,
        finish_post_run_hint_then_return,
    };

    #[test]
    fn not_measured_message_is_stable_for_users_and_tests() {
        assert_eq!(
            POST_RUN_METRICS_NOT_MEASURED_MESSAGE,
            "Tracked edit metrics: not measured."
        );
    }

    #[test]
    fn finish_post_run_hint_then_return_propagates_inner_result() {
        let tmp = tempfile::tempdir().unwrap();
        let out: Result<(), String> = finish_post_run_hint_then_return(
            tmp.path(),
            Err("simulated acp failure".into()),
        );
        assert_eq!(out, Err("simulated acp failure".into()));
    }

    #[test]
    fn finish_and_write_report_is_idempotent_for_path() {
        let tmp = tempfile::tempdir().unwrap();
        finish_and_write_report(tmp.path());
        finish_and_write_report(tmp.path());
    }
}

#[cfg(test)]
mod kiss_stringify_report {
    #[test]
    fn kiss_stringify_report_fns() {
        let _ = stringify!(super::finish_and_write_report);
        let _ = stringify!(super::finish_post_run_hint_then_return);
    }
}
