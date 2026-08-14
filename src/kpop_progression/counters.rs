use std::path::Path;

use crate::kpop_log_protocol::ExperimentLog;

pub fn read_exp_log_text(path: &Path) -> Result<String, String> {
    ExperimentLog::read(path).map(|log| log.as_str().to_string())
}

#[must_use]
pub fn agent_declared_success(text: &str) -> bool {
    ExperimentLog::from_text(text).declares_solved()
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::agent_declared_success;

    #[test]
    fn kiss_cov_agent_declared_success_executes() {
        let text = "## Step 1 — KPop a\n";
        assert!(!agent_declared_success(text));
        assert!(agent_declared_success("## KPOP_SOLVED\n"));
        assert!(agent_declared_success("## KPOP_SOLVED — done\n"));
    }
}
