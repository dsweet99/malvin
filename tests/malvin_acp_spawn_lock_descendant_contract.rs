//! Contract: descendant processes may borrow an ancestor's ACP spawn lock (nested `malvin inspire`).

mod common;

#[cfg(unix)]
use common::fresh_workdir;
#[cfg(unix)]
use malvin::acp::snapshot_pids;
#[cfg(unix)]
use malvin::malvin_sandbox::{
    clear_active_sandbox_session, note_active_sandbox_session,
};
#[cfg(unix)]
use malvin::{active_acp_lock_slot, set_active_acp_lock_slot};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
fn run_descendant_acp_lock_probe(work: &std::path::Path, parent_slot: &str) -> std::process::ExitStatus {
    let exe = std::env::current_exe().expect("current test exe");
    Command::new(&exe)
        .env("MALVIN_ACP_LOCK_DESCENDANT_PROBE", work)
        .env("MALVIN_ACP_LOCK_PARENT_SLOT", parent_slot)
        .args([
            "acp_spawn_lock_descendant_probe_from_env",
            "--exact",
            "--nocapture",
        ])
        .status()
        .expect("spawn descendant probe")
}

/// A descendant process may borrow the ancestor's ACP lock (nested `malvin inspire`).
#[cfg(unix)]
#[test]
fn peer_acp_spawn_lock_allows_descendant_process() {
    if std::env::var_os("MALVIN_ACP_LOCK_DESCENDANT_PROBE").is_some() {
        return;
    }
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_peer_acp_spawn_lock_descendant");
    let baseline = snapshot_pids();
    set_active_acp_lock_slot("parentslot".to_string());
    note_active_sandbox_session(None, baseline, &work).expect("parent acquire");
    let holder_pid = std::process::id().to_string();
    let lock = work
        .join(".malvin")
        .join("acp_spawn")
        .join("parentslot.lock");
    let status = run_descendant_acp_lock_probe(&work, "parentslot");
    assert!(status.success(), "descendant probe failed: {status:?}");
    assert_eq!(
        std::fs::read_to_string(&lock).expect("read lock").trim(),
        holder_pid,
        "ancestor holder pid must remain in lock file after descendant acquire"
    );
    clear_active_sandbox_session();
    assert!(!lock.exists());
    let _ = active_acp_lock_slot;
}

#[test]
fn kiss_cov_malvin_acp_spawn_lock_descendant_contract_symbols() {
    let _ = stringify!(peer_acp_spawn_lock_allows_descendant_process);
}
