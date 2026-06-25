//! Tracks PIDs first observed during an agent sandbox session whose parent chain
//! ties them to malvin or the agent process group, so teardown does not kill unrelated
//! user processes that happen to reparent to init mid-session.

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

use crate::acp::unix_process_group_ps::{ProcRow, INIT_PID, list_proc_rows};

static FIRST_SEEN_PPID: LazyLock<Mutex<HashMap<u32, u32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static AFFILIATED_PIDS: LazyLock<Mutex<HashSet<u32>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn lock_or_recover<T>(mutex: &LazyLock<Mutex<T>>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Clears affiliation state when a sandbox session ends.
pub(crate) fn clear_session_spawn_affiliation() {
    lock_or_recover(&FIRST_SEEN_PPID).clear();
    lock_or_recover(&AFFILIATED_PIDS).clear();
}

#[cfg(test)]
pub(crate) fn clear_session_spawn_affiliation_for_test() {
    clear_session_spawn_affiliation();
}

/// Records first-seen parent links and marks session-affiliated PIDs.
pub(crate) fn refresh_session_spawn_affiliation(
    agent_pgid: Option<u32>,
    baseline: &HashSet<u32>,
) {
    let rows = list_proc_rows().unwrap_or_default();
    let mut first_seen = lock_or_recover(&FIRST_SEEN_PPID);
    for row in &rows {
        if baseline.contains(&row.pid) {
            continue;
        }
        match first_seen.entry(row.pid) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(row.ppid);
            }
            std::collections::hash_map::Entry::Occupied(mut slot) => {
                if *slot.get() == INIT_PID && row.ppid != INIT_PID && row.ppid != row.pid {
                    slot.insert(row.ppid);
                }
            }
        }
    }
    let first_seen_snapshot = first_seen.clone();
    drop(first_seen);

    let mut affiliated = lock_or_recover(&AFFILIATED_PIDS);
    for (&pid, &ppid_at_first) in &first_seen_snapshot {
        if affiliated.contains(&pid) {
            continue;
        }
        let ctx = AffiliationCtx {
            rows: &rows,
            agent_pgid,
            baseline,
            first_seen: &first_seen_snapshot,
        };
        if pid_is_session_affiliated_impl(pid, ppid_at_first, &ctx) {
            affiliated.insert(pid);
        }
    }
}

#[cfg(test)]
pub(crate) struct AffiliationCtx<'a> {
    pub(crate) rows: &'a [ProcRow],
    pub(crate) agent_pgid: Option<u32>,
    pub(crate) baseline: &'a HashSet<u32>,
    pub(crate) first_seen: &'a HashMap<u32, u32>,
}

#[cfg(not(test))]
struct AffiliationCtx<'a> {
    rows: &'a [ProcRow],
    agent_pgid: Option<u32>,
    baseline: &'a HashSet<u32>,
    first_seen: &'a HashMap<u32, u32>,
}

pub(crate) fn note_session_affiliated_pid(pid: u32) {
    lock_or_recover(&AFFILIATED_PIDS).insert(pid);
}

pub(crate) fn is_session_affiliated_pid(pid: u32) -> bool {
    lock_or_recover(&AFFILIATED_PIDS).contains(&pid)
}

pub(crate) fn session_affiliated_or_agent_acp(pid: u32) -> bool {
    is_session_affiliated_pid(pid) || crate::acp::unix_process_group_ps::looks_like_malvin_agent_acp(pid)
}

#[cfg(test)]
pub(crate) fn pid_is_session_affiliated(pid: u32, start_ppid: u32, ctx: &AffiliationCtx<'_>) -> bool {
    pid_is_session_affiliated_impl(pid, start_ppid, ctx)
}

fn pid_is_session_affiliated_impl(pid: u32, start_ppid: u32, ctx: &AffiliationCtx<'_>) -> bool {
    let malvin_pid = std::process::id();
    if pid == malvin_pid {
        return true;
    }
    let mut current_ppid = start_ppid;
    let mut visited = HashSet::new();
    loop {
        if current_ppid == malvin_pid {
            return true;
        }
        if ctx.agent_pgid.is_some_and(|pg| {
            ctx.rows
                .iter()
                .any(|row| row.pid == current_ppid && row.pgid == pg)
        }) {
            return true;
        }
        if !visited.insert(current_ppid) {
            break;
        }
        if current_ppid <= INIT_PID {
            break;
        }
        if ctx.baseline.contains(&current_ppid) && current_ppid != malvin_pid {
            break;
        }
        current_ppid = ctx
            .first_seen
            .get(&current_ppid)
            .copied()
            .or_else(|| {
                ctx.rows
                    .iter()
                    .find(|row| row.pid == current_ppid)
                    .map(|row| row.ppid)
            })
            .unwrap_or(INIT_PID);
    }
    false
}

