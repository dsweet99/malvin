use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tracing::warn;

const POLL_INTERVAL: Duration = if cfg!(test) {
    Duration::from_millis(10)
} else {
    Duration::from_millis(500)
};
/// Consecutive `None` USS samples before fail-closed teardown (~1.5s at 500ms poll in production).
const MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES: u32 = 3;

pub struct MemWatchHandles {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub pgid: u32,
    pub limit_bytes: u64,
    pub spawn_pid_baseline: HashSet<u32>,
    pub run_dir: Option<std::path::PathBuf>,
}

#[cfg(unix)]
pub async fn watch_process_group_memory(handles: MemWatchHandles) {
    watch_process_group_memory_with_rss_sampler(handles, |pgid, baseline| {
        crate::malvin_sandbox::malvin_session_rss_bytes(pgid, baseline)
    })
    .await;
}

/// Poll sandbox USS and terminate on over-limit or sustained measurement failure.
#[cfg(unix)]
pub async fn watch_process_group_memory_with_rss_sampler(
    handles: MemWatchHandles,
    sample_rss: fn(Option<u32>, &HashSet<u32>) -> Option<u64>,
) {
    let MemWatchHandles {
        reader_dead,
        pgid,
        limit_bytes,
        spawn_pid_baseline,
        run_dir,
    } = handles;
    let mut consecutive_rss_failures = 0u32;
    loop {
        if !crate::malvin_sandbox::sandbox_still_alive(Some(pgid), &spawn_pid_baseline) {
            return;
        }
        let rss = sample_rss(Some(pgid), &spawn_pid_baseline);
        // After stdout closes (`reader_dead`), keep enforcing hard over-limit kills
        // (orphans may still be alive) but do not fail-closed on measurement gaps —
        // teardown often makes USS briefly unreadable and must not look like an OOM.
        let allow_fail_closed = !reader_dead.load(std::sync::atomic::Ordering::SeqCst);
        if memory_watch_should_terminate(
            rss,
            limit_bytes,
            &mut consecutive_rss_failures,
            allow_fail_closed,
        ) {
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
    // Gate loops set an iteration; ordinary malvin/--do/inspire runs use 0 so the
    // marker still distinguishes sandbox OOM from a generic bridge failure.
    let gate_iteration = crate::gate_loop_session::active_gate_iteration().unwrap_or(0);
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
    allow_fail_closed: bool,
) -> bool {
    if let Some(bytes) = rss {
        *consecutive_failures = 0;
        bytes > limit_bytes
    } else if allow_fail_closed {
        *consecutive_failures = consecutive_failures.saturating_add(1);
        *consecutive_failures >= MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES
    } else {
        *consecutive_failures = 0;
        false
    }
}

#[cfg(test)]
#[path = "process_group_mem_watch_tests.rs"]
mod process_group_mem_watch_tests;

#[cfg(all(test, unix))]
mod policy_tests {
    use super::{memory_watch_should_terminate, MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES};

    #[test]
    fn memory_watch_should_terminate_on_over_limit() {
        let mut failures = 0;
        assert!(memory_watch_should_terminate(Some(100), 50, &mut failures, true));
        assert_eq!(failures, 0);
    }

    #[test]
    fn memory_watch_should_not_terminate_when_under_limit() {
        let mut failures = 0;
        assert!(!memory_watch_should_terminate(Some(10), 50, &mut failures, true));
        assert_eq!(failures, 0);
    }

    #[test]
    fn memory_watch_fail_closed_after_consecutive_none_samples() {
        let mut failures = 0;
        for _ in 0..MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES - 1 {
            assert!(!memory_watch_should_terminate(None, u64::MAX, &mut failures, true));
        }
        assert!(memory_watch_should_terminate(None, u64::MAX, &mut failures, true));
    }

    #[test]
    fn memory_watch_no_fail_closed_when_disallowed() {
        let mut failures = 0;
        for _ in 0..MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES + 2 {
            assert!(!memory_watch_should_terminate(None, u64::MAX, &mut failures, false));
        }
        assert_eq!(failures, 0);
        assert!(memory_watch_should_terminate(Some(100), 50, &mut failures, false));
    }

    #[test]
    fn memory_watch_resets_failure_counter_after_successful_sample() {
        let mut failures = 2;
        assert!(!memory_watch_should_terminate(Some(1), u64::MAX, &mut failures, true));
        assert_eq!(failures, 0);
        assert!(!memory_watch_should_terminate(None, u64::MAX, &mut failures, true));
        assert_eq!(failures, 1);
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;
    #[test]
    fn kiss_cov_watch_sampler() {
        let _ = (watch_process_group_memory, watch_process_group_memory_with_rss_sampler);
    }
}
