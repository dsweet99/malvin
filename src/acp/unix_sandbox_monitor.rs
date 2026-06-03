#[cfg(unix)]
use std::collections::HashSet;

#[cfg(unix)]
use super::unix_process_group_ps::list_proc_rows;
#[cfg(unix)]
use super::unix_process_group_kill_targets::{descendant_pids, kill_targets_for_teardown};

#[cfg(unix)]
pub fn sandbox_monitor_pids(
    process_group_id: Option<u32>,
    spawn_baseline: &HashSet<u32>,
) -> HashSet<u32> {
    let rows = list_proc_rows().unwrap_or_default();
    let malvin_pid = std::process::id();
    let mut targets = descendant_pids(&HashSet::from([malvin_pid]), &rows);
    targets.extend(kill_targets_for_teardown(
        process_group_id,
        Some(spawn_baseline),
    ));
    targets
}
