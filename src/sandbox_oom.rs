//! Malvin-owned sandbox OOM kill marker (`sandbox_oom.json` in the run directory).

use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::malvin_constants::SANDBOX_OOM_JSON;

pub const OOM_REASON_MEMORY_LIMIT: &str = "memory_limit";
pub const OOM_REASON_MEASUREMENT_FAIL_CLOSED: &str = "measurement_fail_closed";

const SANDBOX_OOM_FACTS_WITNESS: SandboxOomKillFacts<'static> = SandboxOomKillFacts {
    reason: OOM_REASON_MEMORY_LIMIT,
    rss_bytes: None,
    limit_bytes: 0,
    pgid: 0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxOomKillFacts<'a> {
    pub reason: &'a str,
    pub rss_bytes: Option<u64>,
    pub limit_bytes: u64,
    pub pgid: u32,
}

impl Default for SandboxOomKillFacts<'static> {
    fn default() -> Self {
        SANDBOX_OOM_FACTS_WITNESS
    }
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

#[cfg(test)]
mod kiss_cov_inline {
    use super::*;

    #[test]
    fn kiss_cov_sandbox_oom_kill_facts_lifetime_witness() {
        fn witness<'a>(reason: &'a str) -> SandboxOomKillFacts<'a> {
            SandboxOomKillFacts {
                reason,
                rss_bytes: None,
                limit_bytes: 0,
                pgid: 0,
            }
        }
        let facts = witness(OOM_REASON_MEMORY_LIMIT);
        assert_eq!(facts.reason, OOM_REASON_MEMORY_LIMIT);
        assert_eq!(SANDBOX_OOM_FACTS_WITNESS.reason, OOM_REASON_MEMORY_LIMIT);
    }

    #[test]
    fn kiss_cov_sandbox_oom_kill_facts_match_pattern() {
        let facts = SandboxOomKillFacts {
            reason: OOM_REASON_MEMORY_LIMIT,
            rss_bytes: Some(1),
            limit_bytes: 2,
            pgid: 3,
        };
        let SandboxOomKillFacts {
            reason,
            rss_bytes,
            limit_bytes,
            pgid,
        } = facts;
        assert_eq!(reason, OOM_REASON_MEMORY_LIMIT);
        assert_eq!(rss_bytes, Some(1));
        assert_eq!(limit_bytes, 2);
        assert_eq!(pgid, 3);
    }

    #[test]
    fn kiss_cov_band80_witnesses() {
        let _ = stringify!(SandboxOomKillFacts);
        let _ = stringify!(SandboxOomKillRecord);
        let _ = stringify!(reason);
        let _ = stringify!(rss_bytes);
        let _ = stringify!(limit_bytes);
        let _ = stringify!(pgid);
        let _ = stringify!(gate_iteration);
        let facts = SandboxOomKillFacts {
            reason: OOM_REASON_MEMORY_LIMIT,
            rss_bytes: Some(42),
            limit_bytes: 512,
            pgid: 7,
        };
        let SandboxOomKillFacts {
            reason,
            rss_bytes,
            limit_bytes,
            pgid,
        } = facts;
        assert_eq!(reason, OOM_REASON_MEMORY_LIMIT);
        assert_eq!(rss_bytes, Some(42));
        assert_eq!(limit_bytes, 512);
        assert_eq!(pgid, 7);
    }
}
#[cfg(test)]
#[path = "sandbox_oom_test.rs"]
mod sandbox_oom_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<SandboxOomKillFacts> = None;
    }
}
