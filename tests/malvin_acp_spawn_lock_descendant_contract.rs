//! Contract: descendant processes may borrow an ancestor's ACP spawn lock (nested `malvin inspire`).

mod common;

#[cfg(unix)]
use common::fresh_workdir;
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::{Command, ExitStatus};

#[cfg(unix)]
fn seed_acp_spawn_lock(work: &Path) -> (PathBuf, String) {
    let holder_pid = std::process::id().to_string();
    let lock = work.join(".malvin").join("acp_spawn.lock");
    std::fs::create_dir_all(lock.parent().unwrap()).expect("mkdir .malvin");
    std::fs::write(&lock, &holder_pid).expect("write lock");
    (lock, holder_pid)
}

#[cfg(unix)]
fn run_descendant_acp_lock_probe(work: &Path) -> ExitStatus {
    let exe = std::env::current_exe().expect("current test exe");
    Command::new(&exe)
        .env("MALVIN_ACP_LOCK_DESCENDANT_PROBE", work)
        .args([
            "acp_spawn_lock_descendant_probe_from_env",
            "--exact",
            "--nocapture",
        ])
        .status()
        .expect("spawn descendant probe")
}

#[cfg(unix)]
fn assert_lock_holder_unchanged(lock: &Path, holder_pid: &str) {
    assert_eq!(
        std::fs::read_to_string(lock).expect("read lock").trim(),
        holder_pid,
        "ancestor holder pid must remain in lock file after descendant acquire"
    );
}

/// A descendant process may borrow the ancestor's ACP lock (nested `malvin inspire`).
#[cfg(unix)]
#[test]
fn peer_acp_spawn_lock_allows_descendant_process() {
    if std::env::var_os("MALVIN_ACP_LOCK_DESCENDANT_PROBE").is_some() {
        return;
    }
    let work = fresh_workdir("malvin_peer_acp_spawn_lock_descendant");
    let (lock, holder_pid) = seed_acp_spawn_lock(&work);
    let status = run_descendant_acp_lock_probe(&work);
    assert!(status.success(), "descendant probe failed: {status:?}");
    assert_lock_holder_unchanged(&lock, &holder_pid);
    let _ = std::fs::remove_file(&lock);
}

#[test]
fn kiss_cov_malvin_acp_spawn_lock_descendant_contract_symbols() {
}
