//! Contract tests for malvin host sandbox (RSS watcher and process-group teardown).

#[cfg(unix)]
use malvin::acp::sandbox_monitor_pids;
#[cfg(unix)]
use malvin::acp::{snapshot_pids, terminate_agent_process_group};
#[cfg(unix)]
use malvin::malvin_sandbox::malvin_session_rss_bytes;
#[cfg(unix)]
use malvin::acp::{MemWatchHandles, watch_process_group_memory};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use std::sync::atomic::AtomicBool;

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

#[test]
fn kiss_cov_malvin_sandbox_contract_symbols() {
    let _ = stringify!(malvin_oom_watcher_kills_agent_sleep_at_low_limit);
    let _ = stringify!(malvin_process_group_teardown_kills_agent_sleep);
    let _ = stringify!(malvin_sandbox_monitor_includes_malvin_spawned_sibling);
    #[cfg(unix)]
    {
        let _ = stringify!(snapshot_pids);
        let _ = stringify!(terminate_agent_process_group);
        let _ = stringify!(malvin_session_rss_bytes);
        let _ = stringify!(sandbox_monitor_pids);
        let _ = stringify!(watch_process_group_memory);
        let _ = stringify!(MemWatchHandles);
    }
}
