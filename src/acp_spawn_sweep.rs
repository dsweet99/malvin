//! Directory-wide stale lock garbage collection for `.malvin/acp_spawn/`.

use std::path::{Path, PathBuf};

const ACP_SPAWN_LOCK_DIR: &str = "acp_spawn";
pub(crate) const ACP_SPAWN_CHAMBER_GITIGNORE: &str = "*\n";

pub(super) fn acp_spawn_chamber_dir(work_dir: &Path) -> PathBuf {
    work_dir.join(".malvin").join(ACP_SPAWN_LOCK_DIR)
}

pub(super) fn ensure_acp_spawn_chamber_gitignore(chamber: &Path) -> Result<(), String> {
    let path = chamber.join(".gitignore");
    if path.is_file() {
        return Ok(());
    }
    std::fs::write(&path, ACP_SPAWN_CHAMBER_GITIGNORE).map_err(|e| e.to_string())
}

fn is_acp_spawn_lock_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("lock")
}

fn holder_pid_is_stale(contents: &str) -> bool {
    let Some(holder_pid) = contents.trim().parse::<u32>().ok() else {
        return true;
    };
    #[cfg(unix)]
    {
        !crate::acp::pid_alive(holder_pid)
    }
    #[cfg(not(unix))]
    {
        let _ = holder_pid;
        false
    }
}

fn remove_if_stale_acp_lock(path: &Path) -> Result<bool, String> {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    if !holder_pid_is_stale(&contents) {
        return Ok(false);
    }
    std::fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Remove stale lock files under `.malvin/acp_spawn/` (invalid PID or dead holder).
///
/// Live holder PIDs are kept. Returns the number of files removed.
pub fn sweep_stale_acp_spawn_locks(work_dir: &Path) -> Result<usize, String> {
    let chamber = acp_spawn_chamber_dir(work_dir);
    let Ok(entries) = std::fs::read_dir(&chamber) else {
        return Ok(0);
    };
    let mut removed = 0usize;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !is_acp_spawn_lock_file(&path) {
            continue;
        }
        if remove_if_stale_acp_lock(&path)? {
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp_spawn_lock::{
        acquire_acp_spawn_lock_for_slot, release_acp_spawn_lock,
    };

    fn write_lock(chamber: &Path, name: &str, contents: &str) {
        std::fs::write(chamber.join(name), contents).expect("write lock");
    }

    #[cfg(unix)]
    #[test]
    fn sweep_stale_acp_spawn_locks_removes_dead_and_invalid() {
        let work = std::env::temp_dir().join("malvin_acp_spawn_sweep");
        let _ = std::fs::remove_dir_all(&work);
        let chamber = work.join(".malvin/acp_spawn");
        std::fs::create_dir_all(&chamber).expect("mkdir chamber");
        write_lock(&chamber, "dead.lock", "424242");
        write_lock(&chamber, "invalid.lock", "not-a-pid");
        write_lock(&chamber, "live.lock", &std::process::id().to_string());
        let removed = sweep_stale_acp_spawn_locks(&work).expect("sweep");
        assert_eq!(removed, 2, "dead and invalid locks removed");
        assert!(!chamber.join("dead.lock").exists());
        assert!(!chamber.join("invalid.lock").exists());
        assert!(chamber.join("live.lock").exists(), "live lock kept");
        let _ = std::fs::remove_file(chamber.join("live.lock"));
    }

    #[test]
    fn sweep_stale_acp_spawn_locks_noop_on_missing_dir() {
        let work = std::env::temp_dir().join("malvin_acp_spawn_sweep_missing");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).expect("mkdir work");
        assert_eq!(sweep_stale_acp_spawn_locks(&work).expect("sweep"), 0);
    }

    #[cfg(unix)]
    #[test]
    fn sweep_stale_acp_spawn_locks_keeps_concurrent_live_slots() {
        let work = std::env::temp_dir().join("malvin_acp_spawn_sweep_concurrent");
        let _ = std::fs::remove_dir_all(&work);
        let chamber = work.join(".malvin/acp_spawn");
        std::fs::create_dir_all(&chamber).expect("mkdir chamber");
        acquire_acp_spawn_lock_for_slot(&work, "alpha").expect("alpha");
        acquire_acp_spawn_lock_for_slot(&work, "beta").expect("beta");
        write_lock(&chamber, "stale.lock", "424242");
        let removed = sweep_stale_acp_spawn_locks(&work).expect("sweep");
        assert_eq!(removed, 1);
        assert!(chamber.join("alpha.lock").exists());
        assert!(chamber.join("beta.lock").exists());
        release_acp_spawn_lock(&work, "alpha");
        release_acp_spawn_lock(&work, "beta");
    }
}
