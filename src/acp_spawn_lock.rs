//! Cross-process ACP spawn lock: one live agent session per workspace.

use std::path::{Path, PathBuf};
use std::time::Duration;

const ACP_SPAWN_LOCK_NAME: &str = "acp_spawn.lock";
const PEER_LOCK_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[must_use]
pub(crate) fn acp_spawn_lock_path(work_dir: &Path) -> PathBuf {
    work_dir.join(".malvin").join(ACP_SPAWN_LOCK_NAME)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeerLockStatus {
    Clear,
    SelfHeld,
    NestedBlocked,
    PeerHeld,
}

fn peer_lock_status(work_dir: &Path) -> PeerLockStatus {
    let path = acp_spawn_lock_path(work_dir);
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return PeerLockStatus::Clear;
    };
    let Some(holder_pid) = contents.trim().parse::<u32>().ok() else {
        let _ = std::fs::remove_file(&path);
        return PeerLockStatus::Clear;
    };
    let self_pid = std::process::id();
    if holder_pid == self_pid {
        return PeerLockStatus::SelfHeld;
    }
    #[cfg(unix)]
    if crate::acp::pid_alive(holder_pid) {
        if holder_is_ancestor_of_self(holder_pid) {
            return PeerLockStatus::NestedBlocked;
        }
        return PeerLockStatus::PeerHeld;
    }
    #[cfg(not(unix))]
    {
        let _ = holder_pid;
    }
    let _ = std::fs::remove_file(&path);
    PeerLockStatus::Clear
}

fn nested_peer_lock_error(work_dir: &Path, holder_pid: u32) -> String {
    format!(
        "ACP spawn lock held by pid {holder_pid} at {}; nested malvin sessions cannot spawn another agent while a parent ACP session is active in this workspace",
        acp_spawn_lock_path(work_dir).display()
    )
}

fn read_live_peer_lock_holder(work_dir: &Path) -> Option<u32> {
    let path = acp_spawn_lock_path(work_dir);
    let contents = std::fs::read_to_string(&path).ok()?;
    let holder_pid = contents.trim().parse::<u32>().ok()?;
    if holder_pid == std::process::id() {
        return None;
    }
    #[cfg(unix)]
    {
        if crate::acp::pid_alive(holder_pid) {
            return Some(holder_pid);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = holder_pid;
    }
    let _ = std::fs::remove_file(&path);
    None
}

#[cfg(unix)]
fn holder_is_ancestor_of_self(holder_pid: u32) -> bool {
    crate::acp::holder_is_ancestor_of_process(holder_pid)
}

#[cfg(not(unix))]
fn holder_is_ancestor_of_self(_: u32) -> bool {
    false
}

fn announce_peer_lock_wait(holder_pid: u32, work_dir: &Path, last_announced: &mut Option<u32>) {
    if last_announced == &Some(holder_pid) {
        return;
    }
    *last_announced = Some(holder_pid);
    crate::output::print_stdout_line(
        crate::output::MALVIN_WHO,
        &format!(
            "Waiting for malvin session pid {holder_pid} to release ACP spawn lock at {}...",
            acp_spawn_lock_path(work_dir).display()
        ),
    );
}

/// Cross-process guard: one live ACP session per workspace (blocks nested `malvin inspire`).
pub fn assert_no_peer_acp_spawn_lock(work_dir: &Path) -> Result<(), String> {
    match peer_lock_status(work_dir) {
        PeerLockStatus::Clear | PeerLockStatus::SelfHeld => Ok(()),
        PeerLockStatus::NestedBlocked | PeerLockStatus::PeerHeld => {
            let holder_pid = read_live_peer_lock_holder(work_dir).unwrap_or(0);
            Err(nested_peer_lock_error(work_dir, holder_pid))
        }
    }
}

/// Poll until no live peer holds the workspace lock, or fail fast when nested under the holder.
pub async fn wait_for_peer_acp_spawn_lock(work_dir: &Path) -> Result<(), String> {
    let mut last_announced = None;
    loop {
        match peer_lock_status(work_dir) {
            PeerLockStatus::Clear | PeerLockStatus::SelfHeld => return Ok(()),
            PeerLockStatus::NestedBlocked => {
                let holder_pid = read_live_peer_lock_holder(work_dir).unwrap_or(0);
                return Err(nested_peer_lock_error(work_dir, holder_pid));
            }
            PeerLockStatus::PeerHeld => {
                if let Some(holder_pid) = read_live_peer_lock_holder(work_dir) {
                    announce_peer_lock_wait(holder_pid, work_dir, &mut last_announced);
                }
                tokio::time::sleep(PEER_LOCK_POLL_INTERVAL).await;
            }
        }
    }
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

#[must_use]
pub(crate) fn agent_string_is_acp_spawn_lock_held(msg: &str) -> bool {
    msg.contains("ACP spawn lock held")
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
