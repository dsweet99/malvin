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
pub(crate) const MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES: u32 = 3;

pub struct MemWatchHandles {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub pgid: Option<u32>,
    pub limit_bytes: u64,
    pub spawn_pid_baseline: HashSet<u32>,
    pub run_dir: Option<std::path::PathBuf>,
}

#[cfg(unix)]
pub async fn watch_process_group_memory(handles: MemWatchHandles) {
    watch_process_group_memory_with_rss_sampler(handles, |pgid, baseline| {
        crate::agent_process::malvin_sandbox::malvin_session_rss_bytes(pgid, baseline)
    })
    .await;
}

#[cfg(unix)]
pub async fn watch_process_group_memory_with_rss_sampler(
    handles: MemWatchHandles,
    sample_rss: fn(Option<u32>, &HashSet<u32>) -> Option<u64>,
) {
    watch_session_memory_with_rss_sampler(handles, sample_rss).await;
}

#[cfg(unix)]
async fn watch_session_memory_with_rss_sampler(
    handles: MemWatchHandles,
    sample_rss: fn(Option<u32>, &HashSet<u32>) -> Option<u64>,
) {
    let mut consecutive_rss_failures = 0u32;
    loop {
        if let Some(facts) = cgroup_oom_kill_facts(&handles) {
            terminate_for_memory(&handles, facts).await;
            return;
        }
        if !crate::agent_process::malvin_sandbox::sandbox_still_alive(
            handles.pgid,
            &handles.spawn_pid_baseline,
        ) {
            return;
        }
        if kernel_enforces_memory_limit(&handles) {
            consecutive_rss_failures = 0;
            tokio::time::sleep(POLL_INTERVAL).await;
            continue;
        }
        let rss = sample_rss(handles.pgid, &handles.spawn_pid_baseline);
        let allow_fail_closed = !handles
            .reader_dead
            .load(std::sync::atomic::Ordering::SeqCst);
        if memory_watch_should_terminate(
            rss,
            handles.limit_bytes,
            &mut consecutive_rss_failures,
            allow_fail_closed,
        ) {
            terminate_for_memory(&handles, rss_kill_facts(&handles, rss)).await;
            return;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[cfg(unix)]
fn kernel_enforces_memory_limit(handles: &MemWatchHandles) -> bool {
    crate::agent_process::agent_cgroup::active_agent_cgroup_for(handles.pgid).is_some()
}

#[cfg(unix)]
fn cgroup_oom_kill_facts(
    handles: &MemWatchHandles,
) -> Option<crate::agent_process::sandbox_oom::SandboxOomKillFacts<'static>> {
    let cgroup = crate::agent_process::agent_cgroup::active_agent_cgroup_for(handles.pgid)?;
    let oom_kills = cgroup.oom_kill_count();
    if oom_kills == 0 {
        return None;
    }
    warn!(
        oom_kills,
        limit_bytes = handles.limit_bytes,
        pgid = handles.pgid,
        "kernel OOM-killed a process in the malvin agent cgroup; terminating"
    );
    Some(oom_facts(
        handles,
        crate::agent_process::sandbox_oom::OOM_REASON_CGROUP_OOM_KILL,
        None,
    ))
}

#[cfg(unix)]
fn rss_kill_facts(
    handles: &MemWatchHandles,
    rss: Option<u64>,
) -> crate::agent_process::sandbox_oom::SandboxOomKillFacts<'static> {
    let limit_bytes = handles.limit_bytes;
    let pgid = handles.pgid;
    rss.map_or_else(
        || {
            warn!(
                limit_bytes,
                pgid, "malvin sandbox cannot measure memory; terminating (fail-closed)"
            );
            oom_facts(
                handles,
                crate::agent_process::sandbox_oom::OOM_REASON_MEASUREMENT_FAIL_CLOSED,
                None,
            )
        },
        |rss_bytes| {
            warn!(
                rss_bytes,
                limit_bytes, pgid, "malvin sandbox exceeded memory limit; terminating"
            );
            oom_facts(
                handles,
                crate::agent_process::sandbox_oom::OOM_REASON_MEMORY_LIMIT,
                Some(rss_bytes),
            )
        },
    )
}

#[cfg(unix)]
const fn oom_facts(
    handles: &MemWatchHandles,
    reason: &'static str,
    rss_bytes: Option<u64>,
) -> crate::agent_process::sandbox_oom::SandboxOomKillFacts<'static> {
    crate::agent_process::sandbox_oom::SandboxOomKillFacts {
        reason,
        rss_bytes,
        limit_bytes: handles.limit_bytes,
        pgid: match handles.pgid {
            Some(pgid) => pgid,
            None => 0,
        },
    }
}

#[cfg(unix)]
async fn terminate_for_memory(
    handles: &MemWatchHandles,
    facts: crate::agent_process::sandbox_oom::SandboxOomKillFacts<'_>,
) {
    record_sandbox_oom_marker(handles.run_dir.as_deref(), facts);
    crate::agent_process::unix_process_group_teardown::terminate_agent_process_group(
        handles.pgid,
        &handles.spawn_pid_baseline,
    )
    .await;
}

#[cfg(unix)]
fn record_sandbox_oom_marker(
    run_dir: Option<&Path>,
    facts: crate::agent_process::sandbox_oom::SandboxOomKillFacts<'_>,
) {
    let Some(run_dir) = run_dir else {
        return;
    };
    let gate_iteration = crate::gate_loop_session::active_gate_iteration().unwrap_or(0);
    let record =
        crate::agent_process::sandbox_oom::SandboxOomKillRecord::from_facts(gate_iteration, facts);
    if let Err(e) = crate::agent_process::sandbox_oom::record_sandbox_oom_kill(run_dir, record) {
        warn!(error = %e, "failed to write sandbox OOM marker");
    }
}

#[cfg(unix)]
#[allow(clippy::missing_const_for_fn)]
pub(crate) fn memory_watch_should_terminate(
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

#[cfg(test)]
#[path = "process_group_mem_watch_oom_marker_tests.rs"]
mod process_group_mem_watch_oom_marker_tests;

#[cfg(all(test, unix))]
#[path = "process_group_mem_watch_cgroup_tests.rs"]
mod process_group_mem_watch_cgroup_tests;
