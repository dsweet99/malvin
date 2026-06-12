//! Process-tree helpers for sandbox teardown kill-target discovery.

use std::collections::{HashMap, HashSet};

#[path = "session_spawn_affiliation.rs"]
mod session_spawn_affiliation;
#[cfg(test)]
#[path = "session_spawn_affiliation_tests.rs"]
mod session_spawn_affiliation_tests;
pub(crate) use session_spawn_affiliation::{
    clear_session_spawn_affiliation, is_session_affiliated_pid, note_session_affiliated_pid,
    refresh_session_spawn_affiliation, session_affiliated_or_agent_acp,
};
use super::unix_process_group_ps::{
    INIT_PID, ProcRow, host_protected_pids, is_safe_kill_target, list_proc_rows,
    process_group_member_pids,
};

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

pub(crate) fn reparented_init_orphans(baseline: &HashSet<u32>, rows: &[ProcRow]) -> HashSet<u32> {
    let protected = host_protected_pids(rows);
    rows.iter()
        .filter(|row| {
            !baseline.contains(&row.pid)
                && is_safe_kill_target(row.pid, &protected)
                && row.ppid == INIT_PID
                && session_affiliated_or_agent_acp(row.pid)
        })
        .map(|row| row.pid)
        .collect()
}

/// Init-reparented Cursor `agent acp` orphans that baseline amnesty would otherwise skip.
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
                && is_session_affiliated_pid(row.pid)
        })
        .map(|row| row.pid)
        .collect()
}

/// Malvin descendants in malvin's process group spawned after `baseline` (same-PG siblings).
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
                && session_affiliated_or_agent_acp(*pid)
        })
        .collect()
}

pub(crate) fn kill_targets_for_teardown(
    process_group_id: Option<u32>,
    spawn_baseline: Option<&HashSet<u32>>,
) -> HashSet<u32> {
    if let Some(baseline) = spawn_baseline.filter(|b| !b.is_empty()) {
        refresh_session_spawn_affiliation(process_group_id, baseline);
    }
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
