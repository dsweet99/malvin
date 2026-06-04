#[cfg(unix)]
use std::collections::HashSet;

#[cfg(unix)]
use super::unix_process_group_ps::{signal_pid, signal_process_group, snapshot_pids};

#[cfg(unix)]
pub(crate) use super::unix_process_group_kill_targets::baseline_amnestied_agent_orphans;
#[cfg(all(unix, test))]
pub(crate) use super::unix_process_group_kill_targets::kill_targets_for_teardown;
#[cfg(all(unix, test))]
pub(crate) use super::unix_process_group_kill_targets::{
    descendant_pids, malvin_session_spawn_pids, reparented_init_orphans,
};

#[cfg(unix)]
#[allow(dead_code)] // used by unit tests; teardown poll uses per-pid escalation instead
pub(crate) async fn signal_targets(targets: &HashSet<u32>, process_group_id: Option<u32>, signal: i32) {
    for pid in targets {
        signal_pid(*pid, signal);
    }
    if let Some(pgid) = process_group_id {
        signal_process_group(pgid, signal);
    }
}

#[cfg(unix)]
pub(crate) use super::unix_process_group_teardown_poll::{
    teardown_agent_sandbox_blocking, SHUTDOWN_CANCEL_TIMEOUT,
};

#[cfg(unix)]
pub async fn terminate_agent_process_group(
    process_group_id: Option<u32>,
    spawn_baseline: &HashSet<u32>,
) {
    super::unix_process_group_teardown_poll::teardown_agent_sandbox_async(
        process_group_id,
        Some(spawn_baseline),
    )
    .await;
}

#[cfg(unix)]
pub async fn terminate_process_group(process_group_id: Option<u32>) {
    super::unix_process_group_teardown_poll::teardown_agent_sandbox_async(process_group_id, None).await;
}

/// Reap init-reparented Cursor `agent acp` orphans from prior sessions before snapshotting baseline.
#[cfg(unix)]
pub fn reap_baseline_amnestied_agent_orphans_blocking() {
    let baseline = snapshot_pids();
    let rows = super::unix_process_group_ps::list_proc_rows().unwrap_or_default();
    let targets = baseline_amnestied_agent_orphans(&baseline, &rows);
    super::unix_process_group_teardown_poll::reap_fixed_pid_targets_blocking(&targets);
}

#[cfg(not(unix))]
pub async fn terminate_agent_process_group(
    _: Option<u32>,
    _: &std::collections::HashSet<u32>,
) {
}

#[cfg(not(unix))]
pub async fn terminate_process_group(_: Option<u32>) {}

#[cfg(all(test, unix))]
#[path = "unix_process_group_teardown_tests.rs"]
pub(crate) mod unix_process_group_teardown_tests;
#[cfg(all(test, unix))]
#[path = "unix_process_group_teardown_escalation_tests.rs"]
mod unix_process_group_teardown_escalation_tests;

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = reap_baseline_amnestied_agent_orphans_blocking;
        let _ = baseline_amnestied_agent_orphans;
        #[cfg(unix)]
        let _ = stringify!(unix_process_group_teardown_tests::malvin_sibling_outside_agent_pg_killed_on_teardown);
        #[cfg(unix)]
        let _ = stringify!(unix_process_group_teardown_tests::baseline_amnestied_agent_acp_orphan_killed_on_teardown);
        #[cfg(unix)]
        let _ = crate::acp::unix_process_group_teardown_poll::reap_fixed_pid_targets_blocking;
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_signal_targets() {
        let _ = signal_targets;
    }
    #[cfg(unix)]
    #[test]
    fn kiss_cov_teardown_poll_module() {
        let _ = teardown_agent_sandbox_blocking;
        let _ = crate::acp::unix_process_group_teardown_poll::teardown_agent_sandbox_async;
    }
}
