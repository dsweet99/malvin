use std::collections::HashSet;

use crate::agent_process::process_group_rss::MemorySample;
#[cfg(unix)]
use crate::agent_process::process_group_rss::pids_sandbox_sample;
#[cfg(unix)]
use crate::agent_process::sandbox_monitor_pids;

#[cfg(unix)]
#[must_use]
pub fn malvin_session_rss_bytes(
    agent_pgid: Option<u32>,
    session_baseline: &HashSet<u32>,
) -> Option<u64> {
    malvin_session_memory_sample(agent_pgid, session_baseline).map(|sample| sample.bytes)
}

#[cfg(unix)]
#[must_use]
pub fn malvin_session_memory_sample(
    agent_pgid: Option<u32>,
    session_baseline: &HashSet<u32>,
) -> Option<MemorySample> {
    let pids = sandbox_monitor_pids(agent_pgid, session_baseline);
    pids_sandbox_sample(&pids)
}

#[cfg(not(unix))]
#[must_use]
pub fn malvin_session_rss_bytes(_: Option<u32>, _: &HashSet<u32>) -> Option<u64> {
    None
}

#[cfg(not(unix))]
#[must_use]
pub fn malvin_session_memory_sample(_: Option<u32>, _: &HashSet<u32>) -> Option<MemorySample> {
    None
}

#[cfg(unix)]
pub(crate) fn sandbox_still_alive(
    agent_pgid: Option<u32>,
    session_baseline: &HashSet<u32>,
) -> bool {
    crate::agent_process::refresh_session_spawn_affiliation(agent_pgid, session_baseline);
    sandbox_monitor_pids(agent_pgid, session_baseline)
        .into_iter()
        .any(crate::agent_process::pid_alive)
}

#[cfg(not(unix))]
pub(crate) fn sandbox_still_alive(_: Option<u32>, _: &HashSet<u32>) -> bool {
    false
}
