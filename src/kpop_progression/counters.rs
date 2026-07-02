use std::path::Path;

use crate::kpop_log_protocol::ExperimentLog;

use super::mpc_plan::mpc_plan_declares_done;

/// Reads the experiment log at `path` into a string.
///
/// # Errors
///
/// Returns `Err` when the file cannot be read.
pub fn read_exp_log_text(path: &Path) -> Result<String, String> {
    ExperimentLog::read(path).map(|log| log.as_str().to_string())
}

#[must_use]
pub fn count_kpop_entries(text: &str) -> usize {
    ExperimentLog::from_text(text).kpop_step_count()
}

#[must_use]
pub fn count_mbc2_entries(text: &str) -> usize {
    ExperimentLog::from_text(text).mbc2_step_count()
}

#[must_use]
pub fn hypotheses_emitted(text: &str) -> usize {
    let log = ExperimentLog::from_text(text);
    log.kpop_step_count() + log.mbc2_step_count()
}

/// True when the mpc plan file at `path` exists and its trimmed contents are exactly `DONE`.
#[must_use]
pub fn agent_declared_success(path: &Path) -> bool {
    mpc_plan_declares_done(path).unwrap_or(false)
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::{agent_declared_success, count_kpop_entries, count_mbc2_entries, hypotheses_emitted};

    #[test]
    fn kiss_cov_counter_wrappers_execute() {
        let text = "## Step 1 — KPop a\n";
        assert_eq!(count_kpop_entries(text), 1);
        assert_eq!(count_mbc2_entries(text), 0);
        assert_eq!(hypotheses_emitted(text), 1);

        let tmp = tempfile::tempdir().expect("tempdir");
        let mpc = tmp.path().join("mpc_plan.md");
        assert!(!agent_declared_success(&mpc));
        std::fs::write(&mpc, "DONE\n").expect("write");
        assert!(agent_declared_success(&mpc));
    }
}
