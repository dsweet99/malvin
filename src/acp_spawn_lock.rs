//! Cross-process ACP spawn lock: one live agent session per workspace lock slot.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

const ACP_SPAWN_LOCK_DIR: &str = "acp_spawn";

static ACTIVE_ACP_LOCK_SLOT: Mutex<Option<String>> = Mutex::new(None);

/// Records the session-name lock slot for this process (set at entrypoint when `--name` is used).
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
    work_dir
        .join(".malvin")
        .join(ACP_SPAWN_LOCK_DIR)
        .join(format!("{slot}.lock"))
}

/// Cross-process guard: one live agent session per workspace lock slot.
///
/// Blocks unrelated peer processes on the same slot while allowing nested `malvin inspire`
/// from descendant processes of the lock holder, and allowing unrelated slots (different
/// session names) to run concurrently in the same workspace.
pub fn assert_no_peer_acp_spawn_lock(work_dir: &Path) -> Result<(), String> {
    assert_no_peer_acp_spawn_lock_for_slot(work_dir, &active_acp_lock_slot())
}

pub fn assert_no_peer_acp_spawn_lock_for_slot(work_dir: &Path, slot: &str) -> Result<(), String> {
    let path = acp_spawn_lock_path(work_dir, slot);
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return Ok(());
    };
    let Some(holder_pid) = contents.trim().parse::<u32>().ok() else {
        let _ = std::fs::remove_file(&path);
        return Ok(());
    };
    let self_pid = std::process::id();
    if holder_pid == self_pid {
        return Ok(());
    }
    #[cfg(unix)]
    if crate::acp::pid_alive(holder_pid) {
        if crate::acp::is_ancestor_pid(holder_pid, self_pid) {
            return Ok(());
        }
        return Err(format!(
            "ACP spawn lock held by pid {holder_pid} at {}; another malvin session cannot spawn another agent on this lock slot while it is active in this workspace",
            path.display()
        ));
    }
    #[cfg(not(unix))]
    {
        let _ = holder_pid;
    }
    let _ = std::fs::remove_file(&path);
    Ok(())
}

pub(crate) fn acquire_acp_spawn_lock(work_dir: &Path) -> Result<(), String> {
    acquire_acp_spawn_lock_for_slot(work_dir, &active_acp_lock_slot())
}

pub fn acquire_acp_spawn_lock_for_slot(work_dir: &Path, slot: &str) -> Result<(), String> {
    assert_no_peer_acp_spawn_lock_for_slot(work_dir, slot)?;
    let path = acp_spawn_lock_path(work_dir, slot);
    let self_pid = std::process::id();
    if let Ok(contents) = std::fs::read_to_string(&path) {
        if let Ok(holder_pid) = contents.trim().parse::<u32>() {
            #[cfg(unix)]
            if holder_pid != self_pid
                && crate::acp::pid_alive(holder_pid)
                && crate::acp::is_ancestor_pid(holder_pid, self_pid)
            {
                return Ok(());
            }
            #[cfg(not(unix))]
            let _ = holder_pid;
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, self_pid.to_string()).map_err(|e| e.to_string())
}

pub fn release_acp_spawn_lock(work_dir: &Path, slot: &str) {
    let path = acp_spawn_lock_path(work_dir, slot);
    if let Ok(contents) = std::fs::read_to_string(&path) {
        if contents.trim() == std::process::id().to_string() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn acp_spawn_lock_round_trip() {
        let work = std::env::temp_dir().join("malvin_acp_spawn_lock_unit");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).expect("mkdir work");
        let slot = "testslot";
        let lock = acp_spawn_lock_path(&work, slot);
        acquire_acp_spawn_lock_for_slot(&work, slot).expect("acquire");
        assert!(lock.is_file());
        assert_no_peer_acp_spawn_lock_for_slot(&work, slot).expect("self holder");
        release_acp_spawn_lock(&work, slot);
        assert!(!lock.exists());
    }

    #[test]
    fn set_active_acp_lock_slot_used_by_assert_no_peer() {
        set_active_acp_lock_slot("unitslot".into());
        assert_eq!(active_acp_lock_slot(), "unitslot");
        let work = std::env::temp_dir().join("malvin_acp_spawn_lock_active_slot");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).expect("mkdir work");
        assert_no_peer_acp_spawn_lock(&work).expect("no lock file yet");
        acquire_acp_spawn_lock(&work).expect("acquire via active slot");
        assert_no_peer_acp_spawn_lock(&work).expect("self holder");
        release_acp_spawn_lock(&work, "unitslot");
    }

    #[test]
    fn different_acp_lock_slots_do_not_block_each_other() {
        let work = std::env::temp_dir().join("malvin_acp_spawn_lock_slots");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).expect("mkdir work");
        acquire_acp_spawn_lock_for_slot(&work, "alpha").expect("alpha");
        assert_no_peer_acp_spawn_lock_for_slot(&work, "beta").expect("beta slot free");
        acquire_acp_spawn_lock_for_slot(&work, "beta").expect("beta acquire");
        release_acp_spawn_lock(&work, "alpha");
        release_acp_spawn_lock(&work, "beta");
    }

    /// Child probe: `MALVIN_ACP_LOCK_DESCENDANT_PROBE=<workdir>` must pass assert.
    #[cfg(unix)]
    #[test]
    fn acp_spawn_lock_descendant_probe_from_env() {
        let Some(work) = std::env::var_os("MALVIN_ACP_LOCK_DESCENDANT_PROBE") else {
            return;
        };
        let work = Path::new(&work);
        let parent_slot = std::env::var("MALVIN_ACP_LOCK_PARENT_SLOT").unwrap_or_else(|_| {
            format!("pid{}", std::process::id())
        });
        assert_no_peer_acp_spawn_lock_for_slot(work, &parent_slot).expect("descendant must pass");
        acquire_acp_spawn_lock_for_slot(work, &parent_slot).expect("descendant acquire");
        release_acp_spawn_lock(work, &parent_slot);
    }
}
