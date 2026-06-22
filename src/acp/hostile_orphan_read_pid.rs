//! Orphan PID file polling for hostile sandbox regression fixtures.

use std::path::Path;
use std::time::Duration;

use super::process_alive;

#[cfg(unix)]
fn note_fixture_orphan_affiliation(pid: u32, agent_pgid: Option<u32>) {
    let baseline = crate::acp::snapshot_pids();
    crate::acp::refresh_session_spawn_affiliation(agent_pgid, &baseline);
    if agent_pgid.is_some()
        && !crate::acp::unix_process_group_kill_targets::is_session_affiliated_pid(pid)
    {
        // setsid/reparent can outrun a single /proc snapshot; persist affiliation for
        // orphans read while the fixture still names this agent PG.
        crate::acp::unix_process_group_kill_targets::note_session_affiliated_pid(pid);
    }
}

#[cfg(unix)]
fn orphan_pid_if_ready(path: &Path, agent_pgid: Option<u32>) -> Option<u32> {
    let text = std::fs::read_to_string(path).ok()?;
    let pid = text.trim().parse::<u32>().ok()?;
    if !process_alive(pid) {
        return None;
    }
    note_fixture_orphan_affiliation(pid, agent_pgid);
    Some(pid)
}

pub async fn read_orphan_pid(path: &Path, agent_pgid: Option<u32>) -> u32 {
    for _ in 0..50 {
        #[cfg(unix)]
        {
            let baseline = crate::acp::snapshot_pids();
            crate::acp::refresh_session_spawn_affiliation(agent_pgid, &baseline);
        }
        #[cfg(unix)]
        if let Some(pid) = orphan_pid_if_ready(path, agent_pgid) {
            return pid;
        }
        #[cfg(not(unix))]
        if let Ok(text) = std::fs::read_to_string(path) {
            if let Ok(pid) = text.trim().parse::<u32>() {
                if process_alive(pid) {
                    return pid;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!(
        "orphan pid file not written or orphan not alive: {}",
        path.display()
    );
}

#[cfg(test)]
#[path = "hostile_orphan_read_pid_test.rs"]
mod hostile_orphan_read_pid_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = note_fixture_orphan_affiliation;
        let _ = orphan_pid_if_ready;
        let _ = read_orphan_pid;
    }
}
