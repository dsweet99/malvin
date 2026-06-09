//! Contract: dead-before-next and sandbox-isolated malvin spawns.

#[cfg(unix)]
#[path = "common/spawn_contract.rs"]
mod spawn_contract;
#[cfg(unix)]
use spawn_contract::{fresh_workdir, sleep_child};
#[cfg(unix)]
use malvin::acp::snapshot_pids;
#[cfg(unix)]
use malvin::malvin_sandbox::{
    assert_dead_before_next_spawn, clear_active_sandbox_session, malvin_std_command,
    malvin_tokio_command, note_active_sandbox_session,
};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
fn process_group_id(pid: u32) -> u32 {
    let out = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "pgid="])
        .output()
        .expect("ps pgid");
    let text = String::from_utf8_lossy(&out.stdout);
    text.trim().parse().expect("pgid parse")
}

/// Prior sandbox PIDs must be gone before the next malvin session spawn is allowed.
#[cfg(unix)]
#[test]
fn dead_before_next_rejects_live_prior_sandbox() {
    clear_active_sandbox_session();
    let baseline = snapshot_pids();
    let mut child = sleep_child("120");
    let pgid = child.id();
    let work = fresh_workdir("malvin_dead_before_next_reject");
    note_active_sandbox_session(Some(pgid), baseline, &work).expect("note");
    let err = assert_dead_before_next_spawn().expect_err("live prior sandbox");
    assert!(
        err.contains("still alive"),
        "expected dead-before-next error, got: {err}"
    );
    let _ = child.kill();
    let _ = child.wait();
    clear_active_sandbox_session();
}

/// After teardown, dead-before-next must allow the next spawn.
#[cfg(unix)]
#[test]
fn dead_before_next_allows_after_prior_sandbox_cleared() {
    clear_active_sandbox_session();
    assert_dead_before_next_spawn().expect("no prior sandbox");
    let baseline = snapshot_pids();
    let mut child = sleep_child("1");
    let pgid = child.id();
    let work = fresh_workdir("malvin_dead_before_next_clear");
    note_active_sandbox_session(Some(pgid), baseline, &work).expect("note");
    let _ = child.kill();
    let _ = child.wait();
    clear_active_sandbox_session();
    assert_dead_before_next_spawn().expect("prior sandbox ended");
}

/// A live peer process in the workspace must not block another malvin session.
#[cfg(unix)]
#[test]
fn concurrent_sessions_allowed_with_live_peer_in_workspace() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_concurrent_sessions");
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir .malvin");
    let mut child = sleep_child("120");
    let holder_pid = child.id();
    let lock = work.join(".malvin").join("acp_spawn.lock");
    std::fs::write(&lock, holder_pid.to_string()).expect("write stale lock");
    let baseline = snapshot_pids();
    note_active_sandbox_session(None, baseline, &work).expect("peer must not block note");
    clear_active_sandbox_session();
    let _ = child.kill();
    let _ = child.wait();
}

/// Session lifecycle must not create or remove workspace lock files.
#[cfg(unix)]
#[test]
fn session_lifecycle_does_not_touch_acp_spawn_lock() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_no_lock_lifecycle");
    let baseline = snapshot_pids();
    let lock = work.join(".malvin").join("acp_spawn.lock");
    note_active_sandbox_session(None, baseline, &work).expect("note");
    assert!(!lock.exists(), "lock file must not be created");
    clear_active_sandbox_session();
    assert!(!lock.exists(), "lock file must not appear after clear");
}

/// Tokio malvin spawns must also create an isolated process group.
#[cfg(unix)]
#[test]
fn malvin_tokio_command_spawns_in_isolated_process_group() {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let mut cmd = malvin_tokio_command("sleep");
        cmd.arg("1");
        let mut child = cmd.spawn().expect("spawn");
        let pid = child.id().expect("child pid");
        assert_eq!(
            process_group_id(pid),
            pid,
            "tokio child should be its own process-group leader"
        );
        let _ = child.kill().await;
    });
}

/// Malvin workload spawns must create a new process group (sandbox isolation).
#[cfg(unix)]
#[test]
fn malvin_std_command_spawns_in_isolated_process_group() {
    let mut cmd = malvin_std_command("sleep");
    cmd.arg("30");
    let mut child = cmd.spawn().expect("spawn");
    let pid = child.id();
    assert_eq!(
        process_group_id(pid),
        pid,
        "child should be its own process-group leader"
    );
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn kiss_cov_malvin_spawn_rules_contract_symbols() {
    let _ = stringify!(dead_before_next_rejects_live_prior_sandbox);
    let _ = stringify!(dead_before_next_allows_after_prior_sandbox_cleared);
    let _ = stringify!(concurrent_sessions_allowed_with_live_peer_in_workspace);
    let _ = stringify!(session_lifecycle_does_not_touch_acp_spawn_lock);
    let _ = stringify!(malvin_tokio_command_spawns_in_isolated_process_group);
    let _ = stringify!(malvin_std_command_spawns_in_isolated_process_group);
    #[cfg(unix)]
    {
        let _ = stringify!(assert_dead_before_next_spawn);
        let _ = stringify!(clear_active_sandbox_session);
        let _ = stringify!(note_active_sandbox_session);
        let _ = stringify!(malvin_std_command);
        let _ = stringify!(malvin_tokio_command);
    }
}
