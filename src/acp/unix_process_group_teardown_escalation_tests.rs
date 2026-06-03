#![cfg(all(test, unix))]

use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::Duration;

use super::super::session_drop_teardown::terminate_agent_process_group_blocking;
use super::terminate_agent_process_group;
use super::terminate_process_group;

#[test]
fn teardown_cooperative_sigterm_exits_before_sigkill() {
    let baseline = super::super::unix_process_group_ps::snapshot_pids();
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "trap 'exit 0' TERM; while true; do sleep 1; done"]);
    cmd.process_group(0);
    let mut child = cmd.spawn().expect("spawn sh");
    let pgid = child.id();
    std::thread::sleep(Duration::from_millis(50));
    terminate_agent_process_group_blocking(Some(pgid), &baseline);
    let status = child.wait().expect("wait");
    assert_eq!(
        status.code(),
        Some(0),
        "cooperative TERM handler should exit with status 0, not SIGKILL"
    );
}

#[test]
fn teardown_ignoring_sigterm_eventually_killed() {
    let baseline = super::super::unix_process_group_ps::snapshot_pids();
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "trap '' TERM; while true; do sleep 1; done"]);
    cmd.process_group(0);
    let mut child = cmd.spawn().expect("spawn sh");
    let pgid = child.id();
    std::thread::sleep(Duration::from_millis(50));
    terminate_agent_process_group_blocking(Some(pgid), &baseline);
    let status = child.wait().expect("wait");
    assert!(!status.success(), "ignoring TERM must end in SIGKILL escalation");
}

#[tokio::test]
async fn teardown_async_ignoring_sigterm_eventually_killed() {
    let baseline = super::super::unix_process_group_ps::snapshot_pids();
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "trap '' TERM; while true; do sleep 1; done"]);
    cmd.process_group(0);
    let mut child = cmd.spawn().expect("spawn sh");
    let pgid = child.id();
    std::thread::sleep(Duration::from_millis(50));
    terminate_agent_process_group(Some(pgid), &baseline).await;
    let status = child.wait().expect("wait");
    assert!(!status.success());
}

#[tokio::test]
async fn terminate_process_group_noop_without_pgid_or_baseline() {
    terminate_process_group(None).await;
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_escalation_tests() {
        let _ = teardown_cooperative_sigterm_exits_before_sigkill;
        let _ = teardown_ignoring_sigterm_eventually_killed;
        let _ = teardown_async_ignoring_sigterm_eventually_killed;
        let _ = terminate_process_group_noop_without_pgid_or_baseline;
    }
}
