use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tracing::warn;

use super::session_types::AcpSession;

const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// Consecutive `None` RSS samples before fail-closed teardown (~1.5s at 500ms poll).
const MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES: u32 = 3;

pub struct MemWatchHandles {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub pgid: u32,
    pub limit_bytes: u64,
    pub spawn_pid_baseline: HashSet<u32>,
}

pub(crate) fn spawn_process_group_memory_watcher(session: &AcpSession, work_dir: &Path) {
    #[cfg(unix)]
    {
        let limit_bytes = crate::mem_limit_config::load_mem_limit_bytes(work_dir);
        let Some(pgid) = session.0.process_group_id else {
            return;
        };
        let handles = MemWatchHandles {
            reader_dead: Arc::clone(&session.0.reader_dead),
            pgid,
            limit_bytes,
            spawn_pid_baseline: session.0.spawn_pid_baseline.clone(),
        };
        tokio::spawn(async move {
            watch_process_group_memory(handles).await;
        });
    }
    #[cfg(not(unix))]
    {
        let _ = (session, work_dir);
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
    sample_rss: impl Fn(Option<u32>, &HashSet<u32>) -> Option<u64>,
) {
    let MemWatchHandles {
        pgid,
        limit_bytes,
        spawn_pid_baseline,
        ..
    } = handles;
    let mut consecutive_rss_failures = 0u32;
    loop {
        if !crate::malvin_sandbox::sandbox_still_alive(Some(pgid), &spawn_pid_baseline) {
            return;
        }
        let rss = sample_rss(Some(pgid), &spawn_pid_baseline);
        if memory_watch_should_terminate(rss, limit_bytes, &mut consecutive_rss_failures) {
            if let Some(rss_bytes) = rss {
                warn!(
                    rss_bytes,
                    limit_bytes,
                    pgid,
                    "malvin sandbox exceeded memory limit; terminating"
                );
            } else {
                warn!(
                    limit_bytes,
                    pgid,
                    consecutive_failures = consecutive_rss_failures,
                    "malvin sandbox cannot measure memory; terminating (fail-closed)"
                );
            }
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
    use super::{memory_watch_should_terminate, MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES};

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
}
