#[cfg(unix)]
use std::collections::{HashMap, HashSet};

#[cfg(unix)]
use super::unix_process_group_ps::{
    INIT_PID, ProcRow, host_protected_pids, is_safe_kill_target, list_proc_rows,
    process_group_member_pids, signal_pid, signal_process_group,
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

#[cfg(not(unix))]
pub async fn terminate_agent_process_group(
    _: Option<u32>,
    _: &std::collections::HashSet<u32>,
) {
}

#[cfg(not(unix))]
pub async fn terminate_process_group(_: Option<u32>) {}

#[cfg(all(test, unix))]
mod tests {
    use std::collections::HashSet;

    use super::super::unix_process_group_ps::ProcRow;

    #[test]
    fn descendant_pids_walks_child_chain() {
        let rows = vec![
            ProcRow {
                pid: 10,
                pgid: 10,
                ppid: 1,
            },
            ProcRow {
                pid: 11,
                pgid: 10,
                ppid: 10,
            },
            ProcRow {
                pid: 12,
                pgid: 10,
                ppid: 11,
            },
        ];
        let roots = HashSet::from([10]);
        let desc = super::descendant_pids(&roots, &rows);
        assert!(desc.contains(&10));
        assert!(desc.contains(&11));
        assert!(desc.contains(&12));
    }

    #[test]
    fn reparented_init_orphans_matches_setsid_and_double_fork_patterns() {
        let baseline = HashSet::from([10]);
        let rows = vec![
            ProcRow {
                pid: 10,
                pgid: 10,
                ppid: 1,
            },
            ProcRow {
                pid: 99,
                pgid: 99,
                ppid: 1,
            },
            ProcRow {
                pid: 100,
                pgid: 50,
                ppid: 1,
            },
        ];
        let orphans = super::reparented_init_orphans(&baseline, &rows);
        assert!(orphans.contains(&99));
        assert!(orphans.contains(&100));
        assert!(!orphans.contains(&10));
    }

    #[test]
    fn kill_targets_empty_baseline_skips_orphan_scan() {
        let empty = HashSet::new();
        let targets = super::kill_targets_for_teardown(None, Some(&empty));
        assert!(targets.is_empty(), "empty baseline must not scan host orphans");
    }

    #[test]
    fn kill_targets_for_teardown_includes_process_group_members() {
        use std::os::unix::process::CommandExt;
        let baseline = super::super::unix_process_group_ps::snapshot_pids();
        let mut cmd = std::process::Command::new("sleep");
        cmd.arg("30").process_group(0);
        let mut child = cmd.spawn().expect("spawn sleep");
        let pgid = child.id();
        let targets = super::kill_targets_for_teardown(Some(pgid), Some(&baseline));
        assert!(targets.contains(&pgid));
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn kill_targets_for_teardown_excludes_unrelated_concurrent_spawn() {
        use std::os::unix::process::CommandExt;
        let baseline = super::super::unix_process_group_ps::snapshot_pids();
        let mut agent = std::process::Command::new("sleep");
        agent.arg("30").process_group(0);
        let mut agent_child = agent.spawn().expect("spawn agent sleep");
        let agent_pgid = agent_child.id();
        let mut unrelated = std::process::Command::new("sleep");
        unrelated.arg("30");
        let mut unrelated_child = unrelated.spawn().expect("spawn unrelated sleep");
        let unrelated_pid = unrelated_child.id();
        let targets =
            super::kill_targets_for_teardown(Some(agent_pgid), Some(&baseline));
        assert!(targets.contains(&agent_pgid));
        assert!(
            !targets.contains(&unrelated_pid),
            "concurrent host sleep (pid={unrelated_pid}) must not be in teardown kill set"
        );
        let _ = agent_child.kill();
        let _ = unrelated_child.kill();
        let _ = agent_child.wait();
        let _ = unrelated_child.wait();
    }

    #[tokio::test]
    async fn terminate_process_group_kills_sleep_child() {
        use std::os::unix::process::CommandExt;
        let mut cmd = std::process::Command::new("sleep");
        cmd.arg("120").process_group(0);
        let mut child = cmd.spawn().expect("spawn sleep");
        let pgid = child.id();
        super::terminate_process_group(Some(pgid)).await;
        assert_ne!(child.try_wait().expect("wait"), None);
    }

    #[tokio::test]
    async fn terminate_agent_process_group_kills_sleep_child() {
        use std::os::unix::process::CommandExt;
        let baseline = super::super::unix_process_group_ps::snapshot_pids();
        let mut cmd = std::process::Command::new("sleep");
        cmd.arg("120").process_group(0);
        let mut child = cmd.spawn().expect("spawn sleep");
        let pgid = child.id();
        super::terminate_agent_process_group(Some(pgid), &baseline).await;
        assert_ne!(child.try_wait().expect("wait"), None);
    }

    #[tokio::test]
    async fn terminate_process_group_none_is_noop() {
        let _ = super::super::unix_process_group_ps::signal_process_group;
        let _ = super::super::unix_process_group_ps::signal_pid;
        let _ = super::signal_targets;
        super::terminate_process_group(None).await;
    }
}
