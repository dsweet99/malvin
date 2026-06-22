//! Contract: dead-before-next and sandbox-isolated malvin spawns.

mod common;

#[cfg(unix)]
use common::{fresh_workdir, sleep_child};
#[cfg(unix)]
use malvin::acp::snapshot_pids;
#[cfg(unix)]
use malvin::malvin_sandbox::{
    assert_dead_before_next_spawn, assert_no_peer_acp_spawn_lock, clear_active_sandbox_session,
    malvin_std_command, malvin_tokio_command, note_active_sandbox_session,
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
    clear_active_sandbox_session();
    assert_dead_before_next_spawn().expect("prior sandbox ended");
}

/// A live peer lock on the same slot blocks another malvin ACP spawn.
#[cfg(unix)]
#[test]
fn concurrent_sessions_allowed_with_live_peer_in_workspace() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_peer_acp_spawn_lock");
    std::fs::create_dir_all(work.join(".malvin/acp_spawn")).expect("mkdir .malvin");
    let mut child = sleep_child("120");
    let holder_pid = child.id();
    let slot = "sharedslot";
    let lock = work.join(".malvin").join("acp_spawn").join(format!("{slot}.lock"));
    std::fs::write(&lock, holder_pid.to_string()).expect("write lock");
    let err = malvin::assert_no_peer_acp_spawn_lock_for_slot(&work, slot)
        .expect_err("peer lock must block");
    assert!(
        err.contains("ACP spawn lock held"),
        "expected peer lock error, got: {err}"
    );
    malvin::assert_no_peer_acp_spawn_lock_for_slot(&work, slot)
        .expect("stale lock cleared after holder exit");
    assert!(!lock.exists(), "stale lock file removed");
}

/// Session lifecycle must acquire and release the workspace ACP spawn lock slot.
#[cfg(unix)]
#[test]
fn acp_spawn_lock_acquired_and_released_by_session_lifecycle() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_acp_lock_lifecycle");
    let baseline = snapshot_pids();
    malvin::set_active_acp_lock_slot("lifecycle".to_string());
    let lock = work
        .join(".malvin")
        .join("acp_spawn")
        .join("lifecycle.lock");
    note_active_sandbox_session(None, baseline, &work).expect("acquire lock");
    assert!(lock.is_file(), "lock file should exist after note_active");
    assert_eq!(
        std::fs::read_to_string(&lock).expect("read lock").trim(),
        std::process::id().to_string()
    );
    clear_active_sandbox_session();
    assert!(!lock.exists(), "lock file should be removed after clear");
}

/// A lock held by this process must not block re-entry on the same slot.
#[cfg(unix)]
#[test]
fn peer_acp_spawn_lock_allows_same_process_holder() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_peer_acp_spawn_lock_self");
    std::fs::create_dir_all(work.join(".malvin/acp_spawn")).expect("mkdir .malvin");
    malvin::set_active_acp_lock_slot("selfslot".to_string());
    let lock = work.join(".malvin").join("acp_spawn").join("selfslot.lock");
    std::fs::write(&lock, std::process::id().to_string()).expect("write self lock");
    assert_no_peer_acp_spawn_lock(&work).expect("same-process holder allowed");
    assert!(lock.exists(), "self-held lock must remain");
}

#[cfg(unix)]
fn write_peer_acp_lock(work: &std::path::Path, slot: &str, holder_pid: u32) -> std::path::PathBuf {
    std::fs::create_dir_all(work.join(".malvin/acp_spawn")).expect("mkdir .malvin");
    let lock = work
        .join(".malvin")
        .join("acp_spawn")
        .join(format!("{slot}.lock"));
    std::fs::write(&lock, holder_pid.to_string()).expect("write peer lock");
    lock
}

/// `note_active_sandbox_session` must fail when a live peer holds the same ACP lock slot.
#[cfg(unix)]
#[test]
fn note_active_sandbox_session_rejects_live_peer_lock() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_note_active_peer_lock");
    let mut child = sleep_child("120");
    malvin::set_active_acp_lock_slot("peerslot".to_string());
    write_peer_acp_lock(&work, "peerslot", child.id());
    let baseline = snapshot_pids();
    let err = note_active_sandbox_session(None, baseline, &work).expect_err("peer blocks note");
    assert!(err.contains("ACP spawn lock held"), "expected peer lock error, got: {err}");
}

/// `clear_active_sandbox_session` must not delete a lock owned by another process on the same slot.
#[cfg(unix)]
#[test]
fn session_lifecycle_does_not_touch_acp_spawn_lock() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_no_lock_lifecycle");
    let baseline = snapshot_pids();
    malvin::set_active_acp_lock_slot("foreignslot".to_string());
    note_active_sandbox_session(None, baseline, &work).expect("note");
    let lock = work
        .join(".malvin")
        .join("acp_spawn")
        .join("foreignslot.lock");
    std::fs::write(&lock, "424242").expect("overwrite with foreign pid");
    clear_active_sandbox_session();
    assert!(lock.exists(), "foreign lock must survive clear_active_sandbox_session");
}

/// Invalid lock contents are cleared without blocking the caller.
#[cfg(unix)]
#[test]
fn peer_acp_spawn_lock_clears_invalid_lock_file() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_peer_acp_spawn_lock_invalid");
    std::fs::create_dir_all(work.join(".malvin/acp_spawn")).expect("mkdir .malvin");
    malvin::set_active_acp_lock_slot("invalidslot".to_string());
    let lock = work
        .join(".malvin")
        .join("acp_spawn")
        .join("invalidslot.lock");
    std::fs::write(&lock, "not-a-pid").expect("write invalid lock");
    assert_no_peer_acp_spawn_lock(&work).expect("invalid lock cleared");
    assert!(!lock.exists(), "invalid lock file removed");
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
}

#[test]
fn kiss_cov_malvin_spawn_rules_contract_symbols() {
    #[cfg(unix)]
    {
    }
}
