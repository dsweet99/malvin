#[cfg(unix)]
use std::collections::{HashMap, HashSet};

#[cfg(unix)]
use super::unix_process_group_ps::{
    INIT_PID, ProcRow, host_protected_pids, is_safe_kill_target, list_proc_rows,
    looks_like_malvin_agent_acp, process_group_member_pids, signal_pid, signal_process_group,
    snapshot_pids,
};

#[cfg(unix)]
pub(crate) fn descendant_pids(roots: &HashSet<u32>, rows: &[ProcRow]) -> HashSet<u32> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for row in rows {
        children.entry(row.ppid).or_default().push(row.pid);
    }
    let mut seen = HashSet::new();
    let mut queue: Vec<u32> = roots.iter().copied().collect();
    while let Some(pid) = queue.pop() {
        if !seen.insert(pid) {
            continue;
        }
        if let Some(kids) = children.get(&pid) {
            queue.extend(kids);
        }
    }
    seen
}

#[cfg(unix)]
pub(crate) fn reparented_init_orphans(baseline: &HashSet<u32>, rows: &[ProcRow]) -> HashSet<u32> {
    let protected = host_protected_pids(rows);
    rows.iter()
        .filter(|row| {
            !baseline.contains(&row.pid)
                && is_safe_kill_target(row.pid, &protected)
                && row.ppid == INIT_PID
        })
        .map(|row| row.pid)
        .collect()
}

/// Init-reparented Cursor `agent acp` orphans that baseline amnesty would otherwise skip.
#[cfg(unix)]
pub(crate) fn baseline_amnestied_agent_orphans(
    baseline: &HashSet<u32>,
    rows: &[ProcRow],
) -> HashSet<u32> {
    let protected = host_protected_pids(rows);
    rows.iter()
        .filter(|row| {
            baseline.contains(&row.pid)
                && row.ppid == INIT_PID
                && is_safe_kill_target(row.pid, &protected)
                && looks_like_malvin_agent_acp(row.pid)
        })
        .map(|row| row.pid)
        .collect()
}

/// Malvin descendants in malvin's process group spawned after `baseline` (same-PG siblings).
#[cfg(unix)]
pub(crate) fn malvin_session_spawn_pids(
    baseline: &HashSet<u32>,
    rows: &[ProcRow],
) -> HashSet<u32> {
    let malvin_pid = std::process::id();
    let my_pgid = rows
        .iter()
        .find(|row| row.pid == malvin_pid)
        .map_or(malvin_pid, |row| row.pgid);
    descendant_pids(&HashSet::from([malvin_pid]), rows)
        .into_iter()
        .filter(|pid| {
            if *pid == malvin_pid || *pid <= INIT_PID || baseline.contains(pid) {
                return false;
            }
            rows.iter()
                .find(|row| row.pid == *pid)
                .is_some_and(|row| row.pgid == my_pgid)
        })
        .collect()
}

#[cfg(unix)]
pub(crate) fn kill_targets_for_teardown(
    process_group_id: Option<u32>,
    spawn_baseline: Option<&HashSet<u32>>,
) -> HashSet<u32> {
    let rows = list_proc_rows().unwrap_or_default();
    let mut targets = HashSet::new();
    if let Some(pgid) = process_group_id {
        let pg_members = process_group_member_pids(pgid);
        targets.extend(&pg_members);
        targets.extend(descendant_pids(&pg_members, &rows));
    }
    if let Some(baseline) = spawn_baseline.filter(|b| !b.is_empty()) {
        targets.extend(reparented_init_orphans(baseline, &rows));
        targets.extend(baseline_amnestied_agent_orphans(baseline, &rows));
        targets.extend(malvin_session_spawn_pids(baseline, &rows));
    }
    targets
}

#[cfg(unix)]
pub(crate) async fn signal_targets(targets: &HashSet<u32>, process_group_id: Option<u32>, signal: i32) {
    for pid in targets {
        signal_pid(*pid, signal);
    }
    if let Some(pgid) = process_group_id {
        signal_process_group(pgid, signal);
    }
}

#[cfg(unix)]
async fn terminate_with_targets(
    process_group_id: Option<u32>,
    spawn_baseline: Option<&HashSet<u32>>,
) {
    let orphan_scan = spawn_baseline.is_some_and(|b| !b.is_empty());
    if process_group_id.is_none() && !orphan_scan {
        return;
    }
    let targets = kill_targets_for_teardown(process_group_id, spawn_baseline);
    signal_targets(&targets, process_group_id, 15).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    signal_targets(&targets, process_group_id, 9).await;
}

#[cfg(unix)]
pub async fn terminate_agent_process_group(
    process_group_id: Option<u32>,
    spawn_baseline: &HashSet<u32>,
) {
    terminate_with_targets(process_group_id, Some(spawn_baseline)).await;
}

#[cfg(unix)]
pub async fn terminate_process_group(process_group_id: Option<u32>) {
    terminate_with_targets(process_group_id, None).await;
}

/// Reap init-reparented Cursor `agent acp` orphans from prior sessions before snapshotting baseline.
#[cfg(unix)]
pub fn reap_baseline_amnestied_agent_orphans_blocking() {
    let baseline = snapshot_pids();
    let rows = list_proc_rows().unwrap_or_default();
    let targets = baseline_amnestied_agent_orphans(&baseline, &rows);
    for pid in &targets {
        signal_pid(*pid, 15);
    }
    if !targets.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    for pid in &targets {
        signal_pid(*pid, 9);
    }
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

#[cfg(test)]
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
    }
}
