use std::path::Path;
use std::sync::PoisonError;
use std::thread::ThreadId;

use super::IN_PROCESS_ACP_LOCK_SLOTS;

pub(super) enum LockProbe {
    Missing,
    Held,
    Busy(String),
    InProgress,
    Stale,
}

fn in_process_slots() -> std::sync::MutexGuard<'static, std::collections::HashMap<String, ThreadId>>
{
    IN_PROCESS_ACP_LOCK_SLOTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn probe_self_holder(slot: &str, path: &Path) -> LockProbe {
    let in_process = in_process_slots();
    match in_process.get(slot) {
        Some(owner) if *owner == std::thread::current().id() => LockProbe::Held,
        Some(_) => LockProbe::Busy(format!(
            "ACP spawn lock slot {slot:?} already held in this malvin process at {}",
            path.display()
        )),
        None => LockProbe::InProgress,
    }
}

fn probe_foreign_holder(holder_pid: u32, self_pid: u32, path: &Path) -> LockProbe {
    #[cfg(unix)]
    if crate::acp::pid_alive(holder_pid) {
        if crate::acp::is_ancestor_pid(holder_pid, self_pid) {
            return LockProbe::Held;
        }
        return LockProbe::Busy(format!(
            "ACP spawn lock held by pid {holder_pid} at {}; another malvin session cannot spawn another agent on this lock slot while it is active in this workspace",
            path.display()
        ));
    }
    #[cfg(not(unix))]
    let _ = holder_pid;
    LockProbe::Stale
}

pub(super) fn probe_existing_acp_spawn_lock(path: &Path, slot: &str, self_pid: u32) -> LockProbe {
    if !path.is_file() {
        return LockProbe::Missing;
    }
    let Ok(contents) = std::fs::read_to_string(path) else {
        return LockProbe::Stale;
    };
    if contents.trim().is_empty() {
        return LockProbe::InProgress;
    }
    let Ok(holder_pid) = contents.trim().parse::<u32>() else {
        return LockProbe::Stale;
    };
    if holder_pid == self_pid {
        return probe_self_holder(slot, path);
    }
    probe_foreign_holder(holder_pid, self_pid, path)
}
