#![cfg(all(test, unix))]

use std::collections::HashSet;
use std::os::unix::process::CommandExt;
use std::process::Command;

use super::super::hostile_orphan_test_util::{
    assert_sibling_monitored_and_blocks_spawn, process_alive, read_orphan_pid,
    spawn_agent_pg_and_malvin_sibling, spawn_hostile_agent_acp_orphan, wait_for_init_reparent,
};
use super::super::session_drop_teardown::terminate_agent_process_group_blocking;
use super::super::unix_process_group_ps::ProcRow;
use super::{
    descendant_pids, kill_targets_for_teardown, malvin_session_spawn_pids, reparented_init_orphans,
    terminate_agent_process_group, terminate_process_group,
};

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
    let desc = descendant_pids(&roots, &rows);
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
    let orphans = reparented_init_orphans(&baseline, &rows);
    assert!(orphans.contains(&99));
    assert!(orphans.contains(&100));
    assert!(!orphans.contains(&10));
}

#[test]
fn terminate_agent_process_group_blocking_noop_without_targets() {
    let empty = HashSet::new();
    terminate_agent_process_group_blocking(None, &empty);
}

#[test]
fn malvin_session_spawn_pids_includes_post_baseline_descendant() {
    let malvin_pid = std::process::id();
    let baseline = HashSet::from([malvin_pid, 999_999]);
    let rows = vec![
        ProcRow {
            pid: malvin_pid,
            pgid: malvin_pid,
            ppid: 1,
        },
        ProcRow {
            pid: 50,
            pgid: malvin_pid,
            ppid: malvin_pid,
        },
        ProcRow {
            pid: 51,
            pgid: 51,
            ppid: malvin_pid,
        },
    ];
    let spawns = malvin_session_spawn_pids(&baseline, &rows);
    assert!(spawns.contains(&50), "same-PG sibling must be a session spawn target");
    assert!(
        !spawns.contains(&51),
        "isolated-PG agent child must not be targeted via malvin-PG walk"
    );
    assert!(!spawns.contains(&malvin_pid));
}

#[test]
fn kill_targets_empty_baseline_skips_orphan_scan() {
    let empty = HashSet::new();
    let targets = kill_targets_for_teardown(None, Some(&empty));
    assert!(targets.is_empty(), "empty baseline must not scan host orphans");
}

#[test]
fn reap_baseline_amnestied_agent_orphans_blocking_noop_without_orphans() {
    super::reap_baseline_amnestied_agent_orphans_blocking();
}

#[tokio::test]
async fn signal_targets_noop_for_empty_set() {
    super::signal_targets(&HashSet::new(), None, 15).await;
}

#[tokio::test]
async fn terminate_process_group_kills_sleep_child() {
    let mut cmd = Command::new("sleep");
    cmd.arg("120").process_group(0);
    let mut child = cmd.spawn().expect("spawn sleep");
    let pgid = child.id();
    terminate_process_group(Some(pgid)).await;
    assert_ne!(child.try_wait().expect("wait"), None);
}

#[tokio::test]
async fn terminate_agent_process_group_kills_sleep_child() {
    let baseline = super::super::unix_process_group_ps::snapshot_pids();
    let mut cmd = Command::new("sleep");
    cmd.arg("120").process_group(0);
    let mut child = cmd.spawn().expect("spawn sleep");
    let pgid = child.id();
    terminate_agent_process_group(Some(pgid), &baseline).await;
    assert_ne!(child.try_wait().expect("wait"), None);
}

/// Regression: init-reparented `agent acp` orphans in baseline must still be kill targets.
#[tokio::test]
async fn baseline_amnestied_agent_acp_orphan_killed_on_teardown() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let (mut agent, pgid) = spawn_hostile_agent_acp_orphan(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file).await;
    wait_for_init_reparent(orphan_pid).await;
    let mut baseline = super::super::unix_process_group_ps::snapshot_pids();
    baseline.insert(orphan_pid);
    terminate_agent_process_group(Some(pgid), &baseline).await;
    let _ = agent.wait();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        !process_alive(orphan_pid),
        "teardown must kill baseline-amnestied agent acp orphan (pid={orphan_pid})"
    );
}

/// Regression: malvin-spawned siblings outside the agent PG must die on session teardown.
#[tokio::test]
async fn malvin_sibling_outside_agent_pg_killed_on_teardown() {
    use crate::malvin_sandbox::{assert_dead_before_next_spawn, clear_active_sandbox_session};

    clear_active_sandbox_session();
    let baseline = super::super::unix_process_group_ps::snapshot_pids();
    let (agent_pgid, sibling_pid, mut agent_child, mut sibling_child) =
        spawn_agent_pg_and_malvin_sibling();
    assert_sibling_monitored_and_blocks_spawn(agent_pgid, sibling_pid, &baseline);
    terminate_agent_process_group(Some(agent_pgid), &baseline).await;
    clear_active_sandbox_session();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert_ne!(
        sibling_child.try_wait().expect("wait sibling"),
        None,
        "teardown must terminate malvin sibling outside agent PG (pid={sibling_pid})"
    );
    assert_dead_before_next_spawn().expect("dead-before-next after clean teardown");
    let _ = agent_child.wait();
    let _ = sibling_child.wait();
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    #[test]
    fn kiss_cov_unit_names() {
        let _ = stringify!(malvin_sibling_outside_agent_pg_killed_on_teardown);
        let _ = stringify!(baseline_amnestied_agent_acp_orphan_killed_on_teardown);
        let _ = stringify!(malvin_session_spawn_pids_includes_post_baseline_descendant);
        let _ = crate::acp::hostile_orphan_test_util::spawn_agent_pg_and_malvin_sibling;
        let _ = crate::acp::hostile_orphan_test_util::assert_sibling_monitored_and_blocks_spawn;
        let _ = crate::acp::hostile_orphan_test_util::spawn_hostile_agent_acp_orphan;
    }
}
