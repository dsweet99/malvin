//! Malvin-owned sandbox OOM kill marker (`sandbox_oom.json` in the run directory).

use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::malvin_constants::SANDBOX_OOM_JSON;

pub const OOM_REASON_MEMORY_LIMIT: &str = "memory_limit";
pub const OOM_REASON_MEASUREMENT_FAIL_CLOSED: &str = "measurement_fail_closed";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxOomKillFacts<'a> {
    pub reason: &'a str,
    pub rss_bytes: Option<u64>,
    pub limit_bytes: u64,
    pub pgid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxOomKillRecord<'a> {
    pub gate_iteration: usize,
    pub reason: &'a str,
    pub rss_bytes: Option<u64>,
    pub limit_bytes: u64,
    pub pgid: u32,
}

impl<'a> SandboxOomKillRecord<'a> {
    #[must_use]
    pub const fn from_facts(gate_iteration: usize, facts: SandboxOomKillFacts<'a>) -> Self {
        Self {
            gate_iteration,
            reason: facts.reason,
            rss_bytes: facts.rss_bytes,
            limit_bytes: facts.limit_bytes,
            pgid: facts.pgid,
        }
    }
}

/// Writes or overwrites the run-directory OOM marker for the active gate iteration.
///
/// # Errors
///
/// Returns [`std::io::Error`] when the run directory or marker file cannot be written.
pub fn record_sandbox_oom_kill(
    run_dir: &Path,
    record: SandboxOomKillRecord<'_>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(run_dir)?;
    let path = run_dir.join(SANDBOX_OOM_JSON);
    let json = serde_json::json!({
        "gate_iteration": record.gate_iteration,
        "reason": record.reason,
        "rss_bytes": record.rss_bytes,
        "limit_bytes": record.limit_bytes,
        "pgid": record.pgid,
    });
    let text = serde_json::to_string_pretty(&json).map_err(std::io::Error::other)?;
    std::fs::write(path, text)
}

#[must_use]
pub fn gate_iteration_oom_killed(artifacts: &RunArtifacts, prev_gate_iteration: usize) -> bool {
    let path = artifacts.sandbox_oom_json_path();
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    json.get("gate_iteration")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|n| n == u64::try_from(prev_gate_iteration).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_detect_oom_for_gate_iteration() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path()))
            .expect("artifacts");
        record_sandbox_oom_kill(
            &artifacts.run_dir,
            SandboxOomKillRecord::from_facts(
                2,
                SandboxOomKillFacts {
                    reason: OOM_REASON_MEMORY_LIMIT,
                    rss_bytes: Some(999),
                    limit_bytes: 512,
                    pgid: 42,
                },
            ),
        )
        .expect("write");
        assert!(gate_iteration_oom_killed(&artifacts, 2));
        assert!(!gate_iteration_oom_killed(&artifacts, 1));
    }

    #[test]
    fn gate_iteration_oom_killed_false_when_marker_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path()))
            .expect("artifacts");
        assert!(!gate_iteration_oom_killed(&artifacts, 1));
    }
}
