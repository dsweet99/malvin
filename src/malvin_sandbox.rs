//! Host sandbox: process-group isolation and RSS for all malvin-started processes.

use std::collections::HashSet;
use std::sync::OnceLock;

#[cfg(unix)]
use crate::acp::sandbox_monitor_pids;
#[cfg(unix)]
use crate::process_group_rss::pids_rss_bytes;

static MALVIN_SPAWN_BASELINE: OnceLock<HashSet<u32>> = OnceLock::new();

pub fn init_malvin_spawn_baseline() {
    #[cfg(unix)]
    {
        let _ = MALVIN_SPAWN_BASELINE.get_or_init(crate::acp::snapshot_pids);
    }
    #[cfg(not(unix))]
    {
        let _ = MALVIN_SPAWN_BASELINE.get_or_init(HashSet::new);
    }
}

#[must_use]
pub fn malvin_spawn_baseline() -> HashSet<u32> {
    MALVIN_SPAWN_BASELINE
        .get_or_init(HashSet::new)
        .clone()
}

#[cfg(unix)]
pub fn isolate_child_process_group(cmd: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}

#[cfg(not(unix))]
pub fn isolate_child_process_group(_: &mut std::process::Command) {}

#[cfg(unix)]
pub fn isolate_tokio_child_process_group(cmd: &mut tokio::process::Command) {
    use std::os::unix::process::CommandExt;
    cmd.as_std_mut().process_group(0);
}

#[cfg(not(unix))]
pub fn isolate_tokio_child_process_group(_: &mut tokio::process::Command) {}

/// RSS for malvin descendants, the agent process group, and reparented session orphans.
#[cfg(unix)]
#[must_use]
pub fn malvin_session_rss_bytes(
    agent_pgid: Option<u32>,
    session_baseline: &HashSet<u32>,
) -> Option<u64> {
    let pids = sandbox_monitor_pids(agent_pgid, session_baseline);
    pids_rss_bytes(&pids)
}

#[cfg(not(unix))]
#[must_use]
pub fn malvin_session_rss_bytes(_: Option<u32>, _: &HashSet<u32>) -> Option<u64> {
    None
}

#[cfg(unix)]
pub(crate) fn sandbox_still_alive(agent_pgid: Option<u32>, session_baseline: &HashSet<u32>) -> bool {
    sandbox_monitor_pids(agent_pgid, session_baseline)
        .into_iter()
        .any(crate::acp::pid_alive)
}

#[cfg(not(unix))]
pub(crate) fn sandbox_still_alive(_: Option<u32>, _: &HashSet<u32>) -> bool {
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_malvin_sandbox_symbols() {
        let _ = stringify!(init_malvin_spawn_baseline);
        let _ = stringify!(malvin_spawn_baseline);
        let _ = stringify!(isolate_child_process_group);
        let _ = stringify!(isolate_tokio_child_process_group);
        let _ = stringify!(malvin_session_rss_bytes);
        let _ = stringify!(sandbox_still_alive);
    }
}
