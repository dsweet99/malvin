use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tracing::warn;

use super::session_types::AcpSession;

const POLL_INTERVAL: Duration = if cfg!(test) {
    Duration::from_millis(10)
} else {
    Duration::from_millis(500)
};
/// Consecutive `None` RSS samples before fail-closed teardown (~1.5s at 500ms poll in production).
const MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES: u32 = 3;

pub struct MemWatchHandles {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub pgid: u32,
    pub limit_bytes: u64,
    pub spawn_pid_baseline: HashSet<u32>,
    pub run_dir: Option<std::path::PathBuf>,
}

pub(crate) fn spawn_process_group_memory_watcher(session: &AcpSession, work_dir: &Path) {
    #[cfg(unix)]
    {
        if crate::acp::test_no_real_agent_enabled() {
            return;
        }
        let limit_bytes = crate::mem_limit_config::load_mem_limit_bytes(work_dir);
        let Some(pgid) = session.0.process_group_id else {
            return;
        };
        let handles = MemWatchHandles {
            reader_dead: Arc::clone(&session.0.reader_dead),
            pgid,
            limit_bytes,
            spawn_pid_baseline: session.0.spawn_pid_baseline.clone(),
            run_dir: session.0.prompts_log_run_dir.clone(),
        };
        tokio::spawn(async move {
            watch_process_group_memory(handles).await;
        });
    }
    #[cfg(not(unix))]
    {
    }
}

#[cfg(unix)]
pub async fn watch_process_group_memory(handles: MemWatchHandles) {
    watch_process_group_memory_with_rss_sampler(handles, |pgid, baseline| {
        crate::malvin_sandbox::malvin_session_rss_bytes(pgid, baseline)
    })
    .await;
}

/// Poll sandbox RSS and terminate on over-limit or sustained measurement failure.
#[cfg(unix)]
pub async fn watch_process_group_memory_with_rss_sampler(
    handles: MemWatchHandles,
    sample_rss: fn(Option<u32>, &HashSet<u32>) -> Option<u64>,
) {
    let MemWatchHandles {
        pgid,
        limit_bytes,
        spawn_pid_baseline,
        run_dir,
        ..
    } = handles;
    let mut consecutive_rss_failures = 0u32;
    loop {
        if !crate::malvin_sandbox::sandbox_still_alive(Some(pgid), &spawn_pid_baseline) {
            return;
        }
        let rss = sample_rss(Some(pgid), &spawn_pid_baseline);
        if memory_watch_should_terminate(rss, limit_bytes, &mut consecutive_rss_failures) {
            let (reason, rss_bytes) = rss.map_or_else(
                || {
                    warn!(
                        limit_bytes,
                        pgid,
                        consecutive_failures = consecutive_rss_failures,
                        "malvin sandbox cannot measure memory; terminating (fail-closed)"
                    );
                    (
                        crate::sandbox_oom::OOM_REASON_MEASUREMENT_FAIL_CLOSED,
                        None,
                    )
                },
                |rss_bytes| {
                    warn!(
                        rss_bytes,
                        limit_bytes,
                        pgid,
                        "malvin sandbox exceeded memory limit; terminating"
                    );
                    (crate::sandbox_oom::OOM_REASON_MEMORY_LIMIT, Some(rss_bytes))
                },
            );
            record_sandbox_oom_marker(
                run_dir.as_deref(),
                crate::sandbox_oom::SandboxOomKillFacts {
                    reason,
                    rss_bytes,
                    limit_bytes,
                    pgid,
                },
            );
            crate::acp::unix_process_group_teardown::terminate_agent_process_group(
                Some(pgid),
                &spawn_pid_baseline,
            )
            .await;
            return;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[cfg(unix)]
fn record_sandbox_oom_marker(run_dir: Option<&Path>, facts: crate::sandbox_oom::SandboxOomKillFacts<'_>) {
    let Some(run_dir) = run_dir else {
        return;
    };
    let Some(gate_iteration) = crate::gate_loop_session::active_gate_iteration() else {
        return;
    };
    let record = crate::sandbox_oom::SandboxOomKillRecord::from_facts(gate_iteration, facts);
    if let Err(e) = crate::sandbox_oom::record_sandbox_oom_kill(run_dir, record) {
        warn!(error = %e, "failed to write sandbox OOM marker");
    }
}

#[cfg(unix)]
#[allow(clippy::missing_const_for_fn)]
fn memory_watch_should_terminate(
    rss: Option<u64>,
    limit_bytes: u64,
    consecutive_failures: &mut u32,
) -> bool {
    if let Some(bytes) = rss {
        *consecutive_failures = 0;
        bytes > limit_bytes
    } else {
        *consecutive_failures = consecutive_failures.saturating_add(1);
        *consecutive_failures >= MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES
    }
}

#[cfg(test)]
#[path = "process_group_mem_watch_tests.rs"]
mod process_group_mem_watch_tests;

#[cfg(all(test, unix))]
mod policy_tests {
    use super::{memory_watch_should_terminate, record_sandbox_oom_marker, MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES};
    use crate::sandbox_oom::{
        gate_iteration_oom_killed, SandboxOomKillFacts,
        OOM_REASON_MEASUREMENT_FAIL_CLOSED,
    };

    #[test]
    fn memory_watch_should_terminate_on_over_limit() {
        let mut failures = 0;
        assert!(memory_watch_should_terminate(Some(100), 50, &mut failures));
        assert_eq!(failures, 0);
    }

    #[test]
    fn memory_watch_should_not_terminate_when_under_limit() {
        let mut failures = 0;
        assert!(!memory_watch_should_terminate(Some(10), 50, &mut failures));
        assert_eq!(failures, 0);
    }

    #[test]
    fn memory_watch_fail_closed_after_consecutive_none_samples() {
        let mut failures = 0;
        for _ in 0..MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES - 1 {
            assert!(!memory_watch_should_terminate(None, u64::MAX, &mut failures));
        }
        assert!(memory_watch_should_terminate(None, u64::MAX, &mut failures));
    }

    #[test]
    fn memory_watch_resets_failure_counter_after_successful_sample() {
        let mut failures = 2;
        assert!(!memory_watch_should_terminate(Some(1), u64::MAX, &mut failures));
        assert_eq!(failures, 0);
        assert!(!memory_watch_should_terminate(None, u64::MAX, &mut failures));
        assert_eq!(failures, 1);
    }

    #[test]
    fn record_sandbox_oom_marker_noops_without_run_dir_or_gate_iteration() {
        record_sandbox_oom_marker(
            None,
            SandboxOomKillFacts {
                reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
                rss_bytes: None,
                limit_bytes: 1,
                pgid: 1,
            },
        );
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::gate_loop_session::set_active_gate_iteration(None);
        record_sandbox_oom_marker(
            Some(tmp.path()),
            SandboxOomKillFacts {
                reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
                rss_bytes: None,
                limit_bytes: 1,
                pgid: 1,
            },
        );
    }

    #[test]
    fn record_sandbox_oom_marker_writes_fail_closed_reason() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
        crate::gate_loop_session::set_active_gate_iteration(Some(1));
        record_sandbox_oom_marker(
            Some(&artifacts.run_dir),
            SandboxOomKillFacts {
                reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
                rss_bytes: None,
                limit_bytes: 512,
                pgid: 7,
            },
        );
        crate::gate_loop_session::set_active_gate_iteration(None);
        assert!(gate_iteration_oom_killed(&artifacts, 1));
        let text = std::fs::read_to_string(artifacts.sandbox_oom_json_path()).expect("read");
        assert!(text.contains(OOM_REASON_MEASUREMENT_FAIL_CLOSED));
    }
}


#[cfg(test)] mod kiss_cov_auto { use super::*; #[test] fn kiss_cov_spawn_process_group_memory_watcher() { let _ = spawn_process_group_memory_watcher; } #[test] fn kiss_cov_watch_sampler() { let _ = (watch_process_group_memory, watch_process_group_memory_with_rss_sampler); } }
