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
const MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES: u32 = 3;

pub struct MemWatchHandles {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub pgid: u32,
    pub limit_bytes: u64,
    pub spawn_pid_baseline: HashSet<u32>,
    pub run_dir: Option<std::path::PathBuf>,
}

#[cfg(unix)]
pub async fn watch_process_group_memory_with_optional_pgid(handles: MemWatchHandles) {
    watch_session_memory(handles, true).await;
}

#[cfg(unix)]
pub async fn watch_process_group_memory(handles: MemWatchHandles) {
    watch_process_group_memory_with_rss_sampler(handles, |pgid, baseline| {
        crate::malvin_sandbox::malvin_session_rss_bytes(pgid, baseline)
    })
    .await;
}

#[cfg(unix)]
pub async fn watch_process_group_memory_with_rss_sampler(
    handles: MemWatchHandles,
    sample_rss: fn(Option<u32>, &HashSet<u32>) -> Option<u64>,
) {
    watch_session_memory_with_rss_sampler(handles, sample_rss, false).await;
}

#[cfg(unix)]
async fn watch_session_memory(handles: MemWatchHandles, allow_none_pgid: bool) {
    watch_session_memory_with_rss_sampler(
        handles,
        crate::malvin_sandbox::malvin_session_rss_bytes,
        allow_none_pgid,
    )
    .await;
}

#[cfg(unix)]
async fn watch_session_memory_with_rss_sampler(
    handles: MemWatchHandles,
    sample_rss: fn(Option<u32>, &HashSet<u32>) -> Option<u64>,
    allow_none_pgid: bool,
) {
    let MemWatchHandles {
        reader_dead,
        pgid,
        limit_bytes,
        spawn_pid_baseline,
        run_dir,
    } = handles;
    let watch_pgid = if allow_none_pgid && pgid == 0 {
        None
    } else {
        Some(pgid)
    };
    let mut consecutive_rss_failures = 0u32;
    loop {
        if !crate::malvin_sandbox::sandbox_still_alive(watch_pgid, &spawn_pid_baseline) {
            return;
        }
        let rss = sample_rss(watch_pgid, &spawn_pid_baseline);
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
                    (crate::sandbox_oom::OOM_REASON_MEASUREMENT_FAIL_CLOSED, None)
                },
                |rss_bytes| {
                    warn!(
                        rss_bytes,
                        limit_bytes, pgid, "malvin sandbox exceeded memory limit; terminating"
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
                watch_pgid,
                &spawn_pid_baseline,
            )
            .await;
            return;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[cfg(unix)]
fn record_sandbox_oom_marker(
    run_dir: Option<&Path>,
    facts: crate::sandbox_oom::SandboxOomKillFacts<'_>,
) {
    let Some(run_dir) = run_dir else {
        return;
    };
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
#[path = "process_group_mem_watch_policy_tests.rs"]
mod process_group_mem_watch_policy_tests;

#[cfg(test)]
#[path = "process_group_mem_watch_oom_marker_tests.rs"]
mod process_group_mem_watch_oom_marker_tests;
