use std::path::Path;

use crate::kpop_log_protocol::ExperimentLog;

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

#[must_use]
pub fn agent_declared_success(text: &str) -> bool {
    ExperimentLog::from_text(text).declares_solved()
}

#[must_use]
pub fn hypothesis_budget_exhausted(text: &str, max_hypotheses: usize) -> bool {
    hypotheses_emitted(text) >= max_hypotheses
}

/// Returns `Err` when the experiment log has more hypothesis steps than allowed.
///
/// # Errors
///
/// Returns `Err` when the log cannot be read or the step count exceeds `max_hypotheses`.
pub fn check_hypothesis_budget(path: &Path, max_hypotheses: usize) -> Result<(), String> {
    let text = read_exp_log_text(path)?;
    let count = hypotheses_emitted(&text);
    if count > max_hypotheses {
        return Err(format!(
            "experiment log counts {count} hypothesis steps, exceeding --max-hypotheses ({max_hypotheses})"
        ));
    }
    Ok(())
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
        assert!(!agent_declared_success(text));
        assert!(agent_declared_success("## KPOP_SOLVED\n"));
        assert!(agent_declared_success("## KPOP_SOLVED — done\n"));
    }
}
