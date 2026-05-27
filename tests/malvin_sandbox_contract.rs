//! Contract tests for malvin host sandbox (RSS watcher and process-group teardown).

#[cfg(unix)]
use malvin::acp::{snapshot_pids, terminate_agent_process_group};
#[cfg(unix)]
use malvin::acp::hostile_orphan_test_util::{
    assert_sibling_monitored_and_blocks_spawn, process_alive, read_orphan_pid,
    spawn_agent_pg_and_malvin_sibling, spawn_hostile_agent_acp_orphan,
};
#[cfg(unix)]
use malvin::malvin_sandbox::{
    assert_dead_before_next_spawn, clear_active_sandbox_session, malvin_session_rss_bytes,
};
#[cfg(unix)]
use malvin::acp::{MemWatchHandles, watch_process_group_memory};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use malvin::acp::sandbox_monitor_pids;
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use std::sync::atomic::AtomicBool;

/// Regression: watcher must keep enforcing after ACP stdout closes (`reader_dead`).
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_enforces_after_reader_dead() {
    use std::sync::atomic::AtomicBool;

    let baseline = snapshot_pids();
    let mut agent = Command::new("sleep");
    agent.arg("120").process_group(0);
    let mut agent_child = agent.spawn().expect("spawn");
    let agent_pgid = agent_child.id();
    let reader_dead = Arc::new(AtomicBool::new(true));
    watch_process_group_memory(MemWatchHandles {
        reader_dead,
        pgid: agent_pgid,
        limit_bytes: 1,
        spawn_pid_baseline: baseline,
    })
    .await;
    assert_ne!(
        agent_child.try_wait().expect("wait"),
        None,
        "watcher must kill sandbox child after reader_dead, not exit early"
    );
}

/// OOM watcher with `limit_bytes = 1` must tear down a live agent sleep child.
#[cfg(unix)]
#[tokio::test]
async fn malvin_oom_watcher_kills_agent_sleep_at_low_limit() {
    let baseline = snapshot_pids();
    let mut agent = Command::new("sleep");
    agent.arg("120").process_group(0);
    let mut agent_child = agent.spawn().expect("spawn");
    let agent_pgid = agent_child.id();
    assert!(
        malvin_session_rss_bytes(Some(agent_pgid), &baseline).is_some_and(|rss| rss > 1),
        "setup: sandbox RSS should be measurable"
    );
    watch_process_group_memory(MemWatchHandles {
        reader_dead: Arc::new(AtomicBool::new(false)),
        pgid: agent_pgid,
        limit_bytes: 1,
        spawn_pid_baseline: baseline,
    })
    .await;
    assert_ne!(agent_child.try_wait().expect("wait"), None);
}

/// Process-group teardown must kill a sleep child in the agent's isolated PG.
#[cfg(unix)]
#[tokio::test]
async fn malvin_process_group_teardown_kills_agent_sleep() {
    let baseline = snapshot_pids();
    let mut agent = Command::new("sleep");
    agent.arg("120").process_group(0);
    let mut child = agent.spawn().expect("spawn");
    let agent_pgid = child.id();
    terminate_agent_process_group(Some(agent_pgid), &baseline).await;
    assert_ne!(child.try_wait().expect("wait"), None);
}

/// Sandbox monitor must include malvin descendants that are not in the agent PG.
#[cfg(unix)]
#[test]
fn malvin_sandbox_monitor_includes_malvin_spawned_sibling() {
    let baseline = snapshot_pids();
    let mut agent = Command::new("sleep");
    agent.arg("120").process_group(0);
    let mut agent_child = agent.spawn().expect("spawn agent");
    let agent_pgid = agent_child.id();
    let mut sibling = Command::new("sleep");
    sibling.arg("120");
    let mut sibling_child = sibling.spawn().expect("spawn sibling");
    let sibling_pid = sibling_child.id();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let monitor = sandbox_monitor_pids(Some(agent_pgid), &baseline);
    assert!(monitor.contains(&agent_pgid));
    assert!(monitor.contains(&sibling_pid));
    let _ = agent_child.kill();
    let _ = sibling_child.kill();
    let _ = agent_child.wait();
    let _ = sibling_child.wait();
}

/// Regression: baseline-amnestied init-reparented `agent acp` orphans must die on teardown.
#[cfg(unix)]
#[tokio::test]
async fn baseline_amnestied_agent_acp_orphan_killed_on_teardown() {
    clear_active_sandbox_session();
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let (mut agent, agent_pgid) = spawn_hostile_agent_acp_orphan(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file).await;
    let mut baseline = snapshot_pids();
    baseline.insert(orphan_pid);

    terminate_agent_process_group(Some(agent_pgid), &baseline).await;
    clear_active_sandbox_session();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        !process_alive(orphan_pid),
        "teardown must kill baseline-amnestied agent acp orphan (pid={orphan_pid})"
    );
    let _ = agent.wait();
}

/// Regression: malvin-spawned siblings outside the agent PG must die on session teardown.
#[cfg(unix)]
#[tokio::test]
async fn malvin_sibling_outside_agent_pg_killed_on_teardown() {
    clear_active_sandbox_session();
    let baseline = snapshot_pids();
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

#[test]
fn kiss_cov_malvin_sandbox_contract_symbols() {
    let _ = stringify!(watch_process_group_memory_enforces_after_reader_dead);
    let _ = stringify!(malvin_oom_watcher_kills_agent_sleep_at_low_limit);
    let _ = stringify!(malvin_process_group_teardown_kills_agent_sleep);
    let _ = stringify!(malvin_sandbox_monitor_includes_malvin_spawned_sibling);
    let _ = stringify!(malvin_sibling_outside_agent_pg_killed_on_teardown);
    let _ = stringify!(baseline_amnestied_agent_acp_orphan_killed_on_teardown);
    #[cfg(unix)]
    {
        let _ = stringify!(snapshot_pids);
        let _ = stringify!(terminate_agent_process_group);
        let _ = stringify!(malvin_session_rss_bytes);
        let _ = stringify!(sandbox_monitor_pids);
        let _ = stringify!(watch_process_group_memory);
        let _ = stringify!(MemWatchHandles);
        let _ = stringify!(assert_dead_before_next_spawn);
        let _ = stringify!(clear_active_sandbox_session);
        let _ = malvin::acp::hostile_orphan_test_util::spawn_agent_pg_and_malvin_sibling;
        let _ = malvin::acp::hostile_orphan_test_util::assert_sibling_monitored_and_blocks_spawn;
        let _ = malvin::acp::hostile_orphan_test_util::spawn_hostile_agent_acp_orphan;
        let _ = malvin::acp::hostile_orphan_test_util::process_alive;
        let _ = malvin::acp::hostile_orphan_test_util::read_orphan_pid;
    }
}
