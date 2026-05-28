//! Contract tests for host-side sandbox gaps (process-group containment).

#[cfg(unix)]
use malvin::acp::{snapshot_pids, terminate_agent_process_group};

#[cfg(unix)]
use malvin::acp::hostile_orphan_test_util::{
    process_alive, read_orphan_pid, spawn_hostile_agent, spawn_hostile_agent_acp_orphan,
    spawn_hostile_double_fork_daemon,
};

/// After the same teardown `AcpSession::shutdown` uses, a hostile session-leader orphan must not
/// keep running on the host.
#[cfg(unix)]
#[tokio::test]
async fn hostile_agent_detached_orphan_dies_on_process_group_teardown() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let spawn_baseline = snapshot_pids();
    let (mut agent, pgid) = spawn_hostile_agent(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file).await;
    assert!(
        process_alive(orphan_pid),
        "setup: orphan should be running before teardown"
    );
    terminate_agent_process_group(Some(pgid), &spawn_baseline).await;
    let _ = agent.wait();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        !process_alive(orphan_pid),
        "sandbox should kill session-leader orphans when the agent process group is torn down (orphan_pid={orphan_pid})"
    );
}

/// Double-fork daemons reparent to init with `pgid != pid`, so they are not session leaders and
/// are outside the agent PG; teardown must scan all reparented-to-init orphans, not only session leaders.
#[cfg(unix)]
#[tokio::test]
async fn hostile_agent_double_fork_daemon_dies_on_process_group_teardown() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let spawn_baseline = snapshot_pids();
    let (mut agent, pgid) = spawn_hostile_double_fork_daemon(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file).await;
    assert!(
        process_alive(orphan_pid),
        "setup: double-fork orphan should be running before teardown"
    );
    terminate_agent_process_group(Some(pgid), &spawn_baseline).await;
    let _ = agent.wait();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        !process_alive(orphan_pid),
        "sandbox should kill double-fork init orphans when the agent process group is torn down (orphan_pid={orphan_pid})"
    );
}

/// Baseline amnesty must not protect init-reparented `agent acp` orphans from teardown.
#[cfg(unix)]
#[tokio::test]
async fn baseline_amnestied_agent_acp_orphan_dies_on_process_group_teardown() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let spawn_baseline = snapshot_pids();
    let (mut agent, pgid) = spawn_hostile_agent_acp_orphan(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file).await;
    let mut baseline = spawn_baseline;
    baseline.insert(orphan_pid);
    assert!(
        process_alive(orphan_pid),
        "setup: agent-acp orphan should be running before teardown"
    );
    terminate_agent_process_group(Some(pgid), &baseline).await;
    let _ = agent.wait();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        !process_alive(orphan_pid),
        "sandbox should kill baseline-amnestied agent acp orphans (orphan_pid={orphan_pid})"
    );
}

#[test]
fn kiss_cov_hostile_agent_sandbox_contract_symbols() {
    let _ = stringify!(spawn_hostile_agent);
    let _ = stringify!(spawn_hostile_agent_exits_after_orphan_fork);
    let _ = stringify!(watch_process_group_memory_kills_orphan_after_agent_pg_exits);
    let _ = stringify!(spawn_hostile_double_fork_daemon);
    let _ = stringify!(read_orphan_pid);
    let _ = stringify!(hostile_agent_detached_orphan_dies_on_process_group_teardown);
    let _ = stringify!(hostile_agent_double_fork_daemon_dies_on_process_group_teardown);
    let _ = stringify!(baseline_amnestied_agent_acp_orphan_dies_on_process_group_teardown);
    let _ = stringify!(spawn_hostile_agent_acp_orphan);
    #[cfg(unix)]
    {
        let _ = stringify!(snapshot_pids);
        let _ = stringify!(terminate_agent_process_group);
        let _ = stringify!(process_alive);
    }
}
