use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::thread::ThreadId;

use crate::acp_spawn_sweep::{acp_spawn_chamber_dir, ensure_acp_spawn_chamber_gitignore};

#[path = "acp_spawn_lock_peer.rs"]
mod acp_spawn_lock_peer;
#[path = "acp_spawn_lock_probe.rs"]
mod acp_spawn_lock_probe;

pub use acp_spawn_lock_peer::{
    assert_no_peer_acp_spawn_lock, assert_no_peer_acp_spawn_lock_for_slot,
};
use acp_spawn_lock_probe::{LockProbe, probe_existing_acp_spawn_lock};

const ACQUIRE_MAX_ATTEMPTS: usize = 4;

pub(crate) static IN_PROCESS_ACP_LOCK_SLOTS: LazyLock<Mutex<HashMap<String, ThreadId>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static ACTIVE_ACP_LOCK_SLOT: Mutex<Option<String>> = Mutex::new(None);

pub fn set_active_acp_lock_slot(slot: String) {
    if let Ok(mut guard) = ACTIVE_ACP_LOCK_SLOT.lock() {
        *guard = Some(slot);
    }
}

#[must_use]
pub fn active_acp_lock_slot() -> String {
    ACTIVE_ACP_LOCK_SLOT
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| format!("pid{}", std::process::id()))
}

#[must_use]
pub(crate) fn acp_spawn_lock_path(work_dir: &Path, slot: &str) -> PathBuf {
    acp_spawn_chamber_dir(work_dir).join(format!("{slot}.lock"))
}

pub(crate) fn acquire_acp_spawn_lock(work_dir: &Path) -> Result<(), String> {
    acquire_acp_spawn_lock_for_slot(work_dir, &active_acp_lock_slot())
}

fn register_in_process_lock(slot: &str) {
    IN_PROCESS_ACP_LOCK_SLOTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(slot.to_string(), std::thread::current().id());
}

fn try_create_lock_file(path: &Path, slot: &str, self_pid: u32) -> Result<Option<()>, String> {
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => {
            writeln!(file, "{self_pid}").map_err(|e| e.to_string())?;
            register_in_process_lock(slot);
            Ok(Some(()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn acquire_acp_spawn_lock_for_slot(work_dir: &Path, slot: &str) -> Result<(), String> {
    let path = acp_spawn_lock_path(work_dir, slot);
    let self_pid = std::process::id();
    let chamber = acp_spawn_chamber_dir(work_dir);
    std::fs::create_dir_all(&chamber).map_err(|e| e.to_string())?;
    ensure_acp_spawn_chamber_gitignore(&chamber)?;

    for _ in 0..ACQUIRE_MAX_ATTEMPTS {
        match probe_existing_acp_spawn_lock(&path, slot, self_pid) {
            LockProbe::Held => return Ok(()),
            LockProbe::Busy(err) => return Err(err),
            LockProbe::InProgress => {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }
            LockProbe::Stale => {
                let _ = std::fs::remove_file(&path);
            }
            LockProbe::Missing => {}
        }

        if try_create_lock_file(&path, slot, self_pid)?.is_some() {
            return Ok(());
        }
    }

    Err(format!(
        "ACP spawn lock busy at {}; another malvin session cannot spawn another agent on this lock slot while it is active in this workspace",
        path.display()
    ))
}

pub fn release_acp_spawn_lock(work_dir: &Path, slot: &str) {
    IN_PROCESS_ACP_LOCK_SLOTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(slot);
    let path = acp_spawn_lock_path(work_dir, slot);
    if let Ok(contents) = std::fs::read_to_string(&path)
        && contents.trim() == std::process::id().to_string()
    {
        let _ = std::fs::remove_file(&path);
    }
}

#[cfg(unix)]
pub fn wait_for_dir_entry_count(dir: &Path, count: usize) {
    while std::fs::read_dir(dir).map_or(0, std::iter::Iterator::count) < count {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    fn spawn_concurrent_acquire(
        barrier: Arc<Barrier>,
        work_path: PathBuf,
        slot: &'static str,
    ) -> std::thread::JoinHandle<Result<(), String>> {
        std::thread::spawn(move || {
            barrier.wait();
            acquire_acp_spawn_lock_for_slot(&work_path, slot)
        })
    }

    #[cfg(unix)]
    #[test]
    fn acp_spawn_lock_round_trip() {
        crate::test_utils::with_isolated_home(|work| {
            let slot = "testslot";
            let lock = acp_spawn_lock_path(work, slot);
            acquire_acp_spawn_lock_for_slot(work, slot).expect("acquire");
            assert!(lock.is_file());
            assert_no_peer_acp_spawn_lock_for_slot(work, slot).expect("self holder");
            release_acp_spawn_lock(work, slot);
            assert!(!lock.exists());
        });
    }

    #[test]
    fn set_active_acp_lock_slot_used_by_assert_no_peer() {
        set_active_acp_lock_slot("unitslot".into());
        assert_eq!(active_acp_lock_slot(), "unitslot");
        crate::test_utils::with_isolated_home(|work| {
            assert_no_peer_acp_spawn_lock(work).expect("no lock file yet");
            acquire_acp_spawn_lock(work).expect("acquire via active slot");
            assert_no_peer_acp_spawn_lock(work).expect("self holder");
            release_acp_spawn_lock(work, "unitslot");
        });
    }

    #[test]
    fn different_acp_lock_slots_do_not_block_each_other() {
        crate::test_utils::with_isolated_home(|work| {
            acquire_acp_spawn_lock_for_slot(work, "alpha").expect("alpha");
            assert_no_peer_acp_spawn_lock_for_slot(work, "beta").expect("beta slot free");
            acquire_acp_spawn_lock_for_slot(work, "beta").expect("beta acquire");
            release_acp_spawn_lock(work, "alpha");
            release_acp_spawn_lock(work, "beta");
        });
    }

    #[test]
    fn acquire_creates_chamber_gitignore() {
        crate::test_utils::with_isolated_home(|work| {
            acquire_acp_spawn_lock_for_slot(work, "chamber").expect("acquire");
            let gitignore = acp_spawn_chamber_dir(work).join(".gitignore");
            assert!(gitignore.is_file(), "chamber .gitignore should exist");
            assert_eq!(
                std::fs::read_to_string(&gitignore).expect("read"),
                crate::acp_spawn_sweep::ACP_SPAWN_CHAMBER_GITIGNORE
            );
            release_acp_spawn_lock(work, "chamber");
        });
    }

    #[cfg(unix)]
    #[test]
    fn acp_spawn_lock_descendant_probe_from_env() {
        let Some(work) = std::env::var_os("MALVIN_ACP_LOCK_DESCENDANT_PROBE") else {
            return;
        };
        let work = Path::new(&work);
        let parent_slot = std::env::var("MALVIN_ACP_LOCK_PARENT_SLOT")
            .unwrap_or_else(|_| format!("pid{}", std::process::id()));
        assert_no_peer_acp_spawn_lock_for_slot(work, &parent_slot).expect("descendant must pass");
        acquire_acp_spawn_lock_for_slot(work, &parent_slot).expect("descendant acquire");
        release_acp_spawn_lock(work, &parent_slot);
    }

    #[cfg(unix)]
    #[test]
    fn acp_spawn_lock_toctou_probe_from_env() {
        let Some(work) = std::env::var_os("MALVIN_ACP_LOCK_TOCTOU_PROBE") else {
            return;
        };
        let work = Path::new(&work);
        let slot =
            std::env::var("MALVIN_ACP_LOCK_TOCTOU_SLOT").unwrap_or_else(|_| "kpop_toctou".into());
        let ready_dir = std::env::var("MALVIN_ACP_LOCK_TOCTOU_READY_DIR").expect("ready dir");
        let ready_dir = Path::new(&ready_dir);
        std::fs::create_dir_all(ready_dir).expect("ready dir");
        std::fs::write(ready_dir.join(std::process::id().to_string()), b"1").expect("ready");
        wait_for_dir_entry_count(ready_dir, 2);
        acquire_acp_spawn_lock_for_slot(work, &slot).expect("child acquire");
        release_acp_spawn_lock(work, &slot);
    }

    #[test]
    fn acp_spawn_lock_toctou_rejects_concurrent_acquire() {
        crate::test_utils::with_isolated_home(|work| {
            let slot = "kpop_toctou";
            let barrier = Arc::new(Barrier::new(2));
            let work_path = work.to_path_buf();
            let t0 = spawn_concurrent_acquire(Arc::clone(&barrier), work_path.clone(), slot);
            let t1 = spawn_concurrent_acquire(barrier, work_path, slot);
            let r0 = t0.join().expect("thread 0");
            let r1 = t1.join().expect("thread 1");
            let successes = usize::from(r0.is_ok()) + usize::from(r1.is_ok());
            assert_eq!(
                successes, 1,
                "exactly one concurrent acquire must succeed: {r0:?} {r1:?}"
            );
            release_acp_spawn_lock(work, slot);
        });
    }
}
