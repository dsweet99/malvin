#[cfg(unix)]
use std::collections::HashSet;

#[cfg(unix)]
use super::unix_process_group_kill_targets::kill_targets_for_teardown;

#[cfg(unix)]
pub fn sandbox_monitor_pids(
    process_group_id: Option<u32>,
    spawn_baseline: &HashSet<u32>,
) -> HashSet<u32> {
    kill_targets_for_teardown(process_group_id, Some(spawn_baseline))
}

#[cfg(all(test, unix))]
#[path = "unix_sandbox_monitor_tests.rs"]
pub(crate) mod tests;
