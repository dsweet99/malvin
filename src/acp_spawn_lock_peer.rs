use std::path::Path;

use super::{acp_spawn_lock_path, active_acp_lock_slot};

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
