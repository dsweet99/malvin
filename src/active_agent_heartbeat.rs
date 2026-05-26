//! Tracks the live `agent acp` process group for stdout heartbeat stats.

use std::collections::HashSet;
use std::sync::Mutex;

struct ActiveAgentSandbox {
    pgid: u32,
    spawn_baseline: HashSet<u32>,
}

static ACTIVE_AGENT_SANDBOX: Mutex<Vec<ActiveAgentSandbox>> = Mutex::new(Vec::new());

pub(crate) fn register_active_agent_process_group(
    pgid: Option<u32>,
    spawn_baseline: HashSet<u32>,
) {
    let Some(pgid) = pgid.filter(|&id| id != 0) else {
        return;
    };
    ACTIVE_AGENT_SANDBOX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(ActiveAgentSandbox {
            pgid,
            spawn_baseline,
        });
}

pub(crate) fn unregister_active_agent_process_group(pgid: Option<u32>) {
    let Some(pgid) = pgid.filter(|&id| id != 0) else {
        return;
    };
    let mut stack = ACTIVE_AGENT_SANDBOX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if stack.last().is_some_and(|entry| entry.pgid == pgid) {
        stack.pop();
    }
}

#[cfg(test)]
pub(crate) fn clear_active_agent_process_groups_for_test() {
    ACTIVE_AGENT_SANDBOX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}

fn current_active_agent_sandbox() -> Option<ActiveAgentSandbox> {
    ACTIVE_AGENT_SANDBOX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .last()
        .map(|entry| ActiveAgentSandbox {
            pgid: entry.pgid,
            spawn_baseline: entry.spawn_baseline.clone(),
        })
}

pub(crate) struct ActiveAgentStatsSource {
    pub pgid: u32,
    pub spawn_baseline: HashSet<u32>,
}

/// Live agent PG and spawn baseline for sandbox RSS queries (e.g. `current_state`).
#[must_use]
pub fn active_agent_process_group_for_stats() -> Option<ActiveAgentStatsSource> {
    current_active_agent_sandbox().map(|entry| ActiveAgentStatsSource {
        pgid: entry.pgid,
        spawn_baseline: entry.spawn_baseline,
    })
}

/// RSS and process-count suffix for heartbeat payloads, when an agent session is live.
#[must_use]
pub fn active_agent_heartbeat_stats() -> Option<String> {
    let entry = current_active_agent_sandbox()?;
    format_agent_stats(entry.pgid, &entry.spawn_baseline)
}

#[cfg(unix)]
fn format_agent_stats(pgid: u32, spawn_baseline: &HashSet<u32>) -> Option<String> {
    let rss = crate::malvin_sandbox::malvin_session_rss_bytes(Some(pgid), spawn_baseline)?;
    let procs = crate::acp::sandbox_monitor_pids(Some(pgid), spawn_baseline).len();
    let rss_label = crate::log_gc::format_freed(rss);
    Some(format!("sandbox {rss_label} RSS, {procs} procs"))
}

#[cfg(not(unix))]
fn format_agent_stats(_pgid: u32, _: &HashSet<u32>) -> Option<String> {
    None
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_active_agent_heartbeat_privates() {
        let _ = stringify!(ActiveAgentSandbox);
        let _ = stringify!(current_active_agent_sandbox);
        let _ = stringify!(format_agent_stats);
        let _ = stringify!(register_active_agent_process_group);
        let _ = stringify!(unregister_active_agent_process_group);
        let _ = stringify!(active_agent_heartbeat_stats);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        active_agent_heartbeat_stats, clear_active_agent_process_groups_for_test,
        register_active_agent_process_group, unregister_active_agent_process_group,
    };
    use crate::acp::snapshot_pids;

    #[test]
    fn nested_register_unregister_restores_outer_pgid() {
        clear_active_agent_process_groups_for_test();
        let pgid = std::process::id();
        let baseline = snapshot_pids();
        register_active_agent_process_group(Some(pgid), baseline.clone());
        register_active_agent_process_group(Some(pgid), baseline.clone());
        unregister_active_agent_process_group(Some(pgid));
        assert!(active_agent_heartbeat_stats().is_some());
        unregister_active_agent_process_group(Some(pgid));
        assert!(active_agent_heartbeat_stats().is_none());
        clear_active_agent_process_groups_for_test();
    }

    #[cfg(unix)]
    #[test]
    fn active_agent_heartbeat_stats_reports_current_process_group() {
        clear_active_agent_process_groups_for_test();
        let pgid = std::process::id();
        let baseline = snapshot_pids();
        register_active_agent_process_group(Some(pgid), baseline.clone());
        let stats = active_agent_heartbeat_stats().expect("stats");
        assert!(stats.contains("RSS"));
        assert!(stats.contains("procs"));
        let source = super::active_agent_process_group_for_stats().expect("source");
        assert_eq!(source.pgid, pgid);
        assert_eq!(source.spawn_baseline, baseline);
        unregister_active_agent_process_group(Some(pgid));
        clear_active_agent_process_groups_for_test();
    }

    #[test]
    fn active_agent_process_group_for_stats_none_when_unregistered() {
        clear_active_agent_process_groups_for_test();
        assert!(super::active_agent_process_group_for_stats().is_none());
    }
}
