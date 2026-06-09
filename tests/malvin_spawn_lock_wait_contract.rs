//! Contract: concurrent malvin sessions wait for workspace ACP spawn lock release.

#[cfg(unix)]
#[path = "common/spawn_contract.rs"]
mod spawn_contract;
#[cfg(unix)]
use spawn_contract::{fresh_workdir, sleep_child};
#[cfg(unix)]
use malvin::malvin_sandbox::{
    assert_no_peer_acp_spawn_lock, clear_active_sandbox_session, wait_for_peer_acp_spawn_lock,
};
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Child;
#[cfg(unix)]
use std::sync::mpsc::{self, RecvTimeoutError};
#[cfg(unix)]
use std::time::Duration;

#[cfg(unix)]
fn write_peer_lock(work: &Path, child: &Child) -> PathBuf {
    let lock = work.join(".malvin").join("acp_spawn.lock");
    std::fs::write(&lock, child.id().to_string()).expect("write lock");
    lock
}

#[cfg(unix)]
fn spawn_lock_wait_thread(work: PathBuf) -> mpsc::Receiver<Result<(), String>> {
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        done_tx
            .send(rt.block_on(wait_for_peer_acp_spawn_lock(&work)))
            .expect("send wait result");
    });
    done_rx
}

#[cfg(unix)]
fn recv_wait_result(done_rx: &mpsc::Receiver<Result<(), String>>) {
    match done_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => {}
        Ok(Err(err)) => panic!("wait for peer lock failed: {err}"),
        Err(RecvTimeoutError::Timeout) => panic!("wait did not finish after holder exited"),
        Err(RecvTimeoutError::Disconnected) => panic!("wait thread exited without sending result"),
    }
}

/// Concurrent outer sessions wait until a live peer releases the workspace lock.
#[cfg(unix)]
#[test]
fn wait_for_peer_acp_spawn_lock_waits_for_unrelated_holder() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_wait_peer_acp_spawn_lock");
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir .malvin");
    let mut child = sleep_child("120");
    let _lock = write_peer_lock(&work, &child);
    let done_rx = spawn_lock_wait_thread(work.clone());
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        child.try_wait().expect("try_wait").is_none(),
        "holder process should stay alive during wait"
    );
    assert_eq!(
        done_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty),
        "wait should block while an unrelated live peer holds the lock"
    );
    let _ = child.kill();
    let _ = child.wait();
    recv_wait_result(&done_rx);
    assert_no_peer_acp_spawn_lock(&work).expect("stale lock cleared after holder exit");
}

#[test]
fn kiss_cov_malvin_spawn_lock_wait_contract_symbols() {
    let _ = stringify!(wait_for_peer_acp_spawn_lock_waits_for_unrelated_holder);
}
