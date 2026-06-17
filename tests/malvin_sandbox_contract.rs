//! Contract tests for malvin host sandbox (RSS watcher and process-group teardown).

#[cfg(unix)]
use malvin::acp::{snapshot_pids, terminate_agent_process_group};
#[cfg(unix)]
use malvin::acp::hostile_orphan_test_util::{
    assert_sibling_monitored_and_blocks_spawn, cleanup_user_coincidental_test,
    process_alive, setup_user_init_reparented_daemon, spawn_agent_pg_and_malvin_sibling,
    spawn_isolated_agent_sleep, spawn_user_shell_cooperator,
};
#[cfg(target_os = "linux")]
use malvin::acp::hostile_orphan_test_util::{
    read_orphan_pid, spawn_hostile_agent_acp_orphan, wait_for_init_reparent,
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
        run_dir: None,
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
        "setup: sandbox USS should be measurable"
    );
    watch_process_group_memory(MemWatchHandles {
        reader_dead: Arc::new(AtomicBool::new(false)),
        pgid: agent_pgid,
        limit_bytes: 1,
        spawn_pid_baseline: baseline,
        run_dir: None,
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
///
/// Linux-only: `looks_like_malvin_agent_acp` reads `/proc/{pid}/environ`; init-reparent timing
/// is asserted via `wait_for_init_reparent` (see `unix_process_group_teardown_tests.rs`).
#[cfg(target_os = "linux")]
#[tokio::test]
async fn baseline_amnestied_agent_acp_orphan_killed_on_teardown() {
    clear_active_sandbox_session();
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let (mut agent, agent_pgid) = spawn_hostile_agent_acp_orphan(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file, Some(agent_pgid)).await;
    wait_for_init_reparent(orphan_pid).await;
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

/// Regression: init-reparented user daemons started mid-session must survive agent teardown.
#[cfg(unix)]
#[tokio::test]
async fn user_coincidental_init_orphan_survives_agent_teardown() {
    clear_active_sandbox_session();
    let tmp = tempfile::tempdir().expect("tempdir");
    let user_daemon_pid_file = tmp.path().join("user_daemon.pid");
    let (user_shell, mut user_shell_stdin) = spawn_user_shell_cooperator();
    let baseline = snapshot_pids();
    let (agent_child, agent_pgid) = spawn_isolated_agent_sleep();
    let user_daemon_pid =
        setup_user_init_reparented_daemon(&mut user_shell_stdin, &user_daemon_pid_file).await;
    terminate_agent_process_group(Some(agent_pgid), &baseline).await;
    clear_active_sandbox_session();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        process_alive(user_daemon_pid),
        "teardown must not kill unrelated user daemon (pid={user_daemon_pid})"
    );
    cleanup_user_coincidental_test(user_daemon_pid, user_shell, agent_child);
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
    let _ = watch_process_group_memory_enforces_after_reader_dead;
    let _ = malvin_oom_watcher_kills_agent_sleep_at_low_limit;
    let _ = malvin_process_group_teardown_kills_agent_sleep;
    let _ = malvin_sandbox_monitor_includes_malvin_spawned_sibling;
    let _ = malvin_sibling_outside_agent_pg_killed_on_teardown;
    let _ = user_coincidental_init_orphan_survives_agent_teardown;
    #[cfg(target_os = "linux")]
    {
        let _ = baseline_amnestied_agent_acp_orphan_killed_on_teardown;
        let _ = spawn_hostile_agent_acp_orphan;
        let _ = read_orphan_pid;
        let _ = wait_for_init_reparent;
    }
    #[cfg(unix)]
    {
        let _ = snapshot_pids;
        let _ = terminate_agent_process_group;
        let _ = malvin_session_rss_bytes;
        let _ = sandbox_monitor_pids;
        let _ = watch_process_group_memory;
        let _ = std::any::type_name::<MemWatchHandles>;
        let _ = assert_dead_before_next_spawn;
        let _ = clear_active_sandbox_session;
        let _ = spawn_agent_pg_and_malvin_sibling;
        let _ = assert_sibling_monitored_and_blocks_spawn;
        let _ = spawn_user_shell_cooperator;
        let _ = spawn_isolated_agent_sleep;
        let _ = setup_user_init_reparented_daemon;
        let _ = cleanup_user_coincidental_test;
        let _ = process_alive;
    }
}
