mod common;

#[cfg(unix)]
use common::{enable_test_fast_teardown, test_wait_until_async};

#[cfg(unix)]
use malvin::acp::{snapshot_pids, terminate_agent_process_group};

#[cfg(unix)]
use malvin::acp::hostile_orphan_test_util::{
    process_alive, read_orphan_pid, spawn_hostile_agent,
    spawn_hostile_agent_exits_after_orphan_fork, spawn_hostile_double_fork_daemon,
};
#[cfg(target_os = "linux")]
use malvin::acp::hostile_orphan_test_util::{
    spawn_hostile_agent_acp_orphan, wait_for_init_reparent,
};

#[cfg(unix)]
#[tokio::test]
async fn hostile_agent_detached_orphan_dies_on_process_group_teardown() {
    enable_test_fast_teardown();
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let spawn_baseline = snapshot_pids();
    let (mut agent, pgid) = spawn_hostile_agent(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file, Some(pgid)).await;
    assert!(
        process_alive(orphan_pid),
        "setup: orphan should be running before teardown"
    );
    terminate_agent_process_group(Some(pgid), &spawn_baseline).await;
    let _ = agent.wait();
    assert!(
        test_wait_until_async(|| !process_alive(orphan_pid)).await,
        "sandbox should kill session-leader orphans when the agent process group is torn down (orphan_pid={orphan_pid})"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn hostile_agent_double_fork_daemon_dies_on_process_group_teardown() {
    enable_test_fast_teardown();
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let spawn_baseline = snapshot_pids();
    let (mut agent, pgid) = spawn_hostile_double_fork_daemon(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file, Some(pgid)).await;
    assert!(
        process_alive(orphan_pid),
        "setup: double-fork orphan should be running before teardown"
    );
    terminate_agent_process_group(Some(pgid), &spawn_baseline).await;
    let _ = agent.wait();
    assert!(
        test_wait_until_async(|| !process_alive(orphan_pid)).await,
        "sandbox should kill double-fork init orphans when the agent process group is torn down (orphan_pid={orphan_pid})"
    );
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn baseline_amnestied_agent_acp_orphan_dies_on_process_group_teardown() {
    enable_test_fast_teardown();
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let spawn_baseline = snapshot_pids();
    let (mut agent, pgid) = spawn_hostile_agent_acp_orphan(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file, Some(pgid)).await;
    wait_for_init_reparent(orphan_pid).await;
    let mut baseline = spawn_baseline;
    baseline.insert(orphan_pid);
    assert!(
        process_alive(orphan_pid),
        "setup: agent-acp orphan should be running before teardown"
    );
    terminate_agent_process_group(Some(pgid), &baseline).await;
    let _ = agent.wait();
    assert!(
        test_wait_until_async(|| !process_alive(orphan_pid)).await,
        "sandbox should kill baseline-amnestied agent acp orphans (orphan_pid={orphan_pid})"
    );
}

#[test]
fn kiss_cov_hostile_agent_sandbox_contract_symbols() {
    #[cfg(unix)]
    {
        let _ = spawn_hostile_agent;
        let _ = spawn_hostile_agent_exits_after_orphan_fork;
        let _ = spawn_hostile_double_fork_daemon;
        let _ = read_orphan_pid;
        let _ = hostile_agent_detached_orphan_dies_on_process_group_teardown;
        let _ = hostile_agent_double_fork_daemon_dies_on_process_group_teardown;
        let _ = snapshot_pids;
        let _ = terminate_agent_process_group;
        let _ = process_alive;
    }
    #[cfg(target_os = "linux")]
    {
        let _ = baseline_amnestied_agent_acp_orphan_dies_on_process_group_teardown;
        let _ = spawn_hostile_agent_acp_orphan;
    }
}
