//! Cross-process ACP spawn lock: one live agent session per workspace.

use std::path::{Path, PathBuf};

const ACP_SPAWN_LOCK_NAME: &str = "acp_spawn.lock";

#[must_use]
pub(crate) fn acp_spawn_lock_path(work_dir: &Path) -> PathBuf {
    work_dir.join(".malvin").join(ACP_SPAWN_LOCK_NAME)
}

/// Cross-process guard: one live ACP session per workspace (blocks nested `malvin inspire`).
pub fn assert_no_peer_acp_spawn_lock(work_dir: &Path) -> Result<(), String> {
    let path = acp_spawn_lock_path(work_dir);
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
        return Err(format!(
            "ACP spawn lock held by pid {holder_pid} at {}; nested malvin sessions cannot spawn another agent while a parent ACP session is active in this workspace",
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
    assert_no_peer_acp_spawn_lock(work_dir)?;
    let path = acp_spawn_lock_path(work_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, std::process::id().to_string()).map_err(|e| e.to_string())
}

pub(crate) fn release_acp_spawn_lock(work_dir: &Path) {
    let path = acp_spawn_lock_path(work_dir);
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
        let lock = acp_spawn_lock_path(&work);
        acquire_acp_spawn_lock(&work).expect("acquire");
        assert!(lock.is_file());
        assert_no_peer_acp_spawn_lock(&work).expect("self holder");
        release_acp_spawn_lock(&work);
        assert!(!lock.exists());
    }
}
