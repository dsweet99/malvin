#[cfg(unix)]
use std::collections::HashSet;
#[cfg(unix)]
use std::process::Stdio;

#[cfg(unix)]
#[path = "unix_process_group_ps_proc.rs"]
mod unix_process_group_ps_proc;

#[cfg(unix)]
pub(crate) const INIT_PID: u32 = 1;

#[cfg(unix)]
pub(crate) struct ProcRow {
    pub pid: u32,
    pub pgid: u32,
    pub ppid: u32,
}

#[cfg(unix)]
pub fn snapshot_pids() -> HashSet<u32> {
    if let Some(pids) = unix_process_group_ps_proc::snapshot_pids_from_proc()
        && proc_pid_snapshot_is_usable(&pids)
    {
        return pids;
    }
    list_pids_from_ps().unwrap_or_default()
}

#[cfg(unix)]
fn proc_pid_snapshot_is_usable(pids: &HashSet<u32>) -> bool {
    pids.contains(&std::process::id())
}

#[cfg(unix)]
pub(crate) fn list_pids_from_ps() -> Option<HashSet<u32>> {
    let out = std::process::Command::new("ps")
        .args(["-ax", "-o", "pid="])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    Some(parse_pid_list(&out.stdout))
}

#[cfg(unix)]
pub(crate) fn list_proc_rows() -> Option<Vec<ProcRow>> {
    if let Some(rows) = unix_process_group_ps_proc::list_proc_rows_from_proc()
        && proc_row_snapshot_is_usable(&rows)
    {
        return Some(rows);
    }
    let out = std::process::Command::new("ps")
        .args(["-ax", "-o", "pid=", "-o", "pgid=", "-o", "ppid="])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    Some(parse_proc_rows(&out.stdout))
}

#[cfg(unix)]
fn proc_row_snapshot_is_usable(rows: &[ProcRow]) -> bool {
    rows.iter().any(|row| row.pid == std::process::id())
}

#[cfg(unix)]
pub(crate) fn parse_pid_list(bytes: &[u8]) -> HashSet<u32> {
    let mut pids = HashSet::new();
    for line in bytes.split(|b| *b == b'\n') {
        if let Ok(text) = std::str::from_utf8(line)
            && let Ok(pid) = text.trim().parse::<u32>()
        {
            pids.insert(pid);
        }
    }
    pids
}

#[cfg(unix)]
pub(crate) fn parse_u32_field(text: &str) -> Option<u32> {
    text.trim().parse::<u32>().ok()
}

#[cfg(unix)]
pub(crate) fn parse_proc_rows(bytes: &[u8]) -> Vec<ProcRow> {
    let mut rows = Vec::new();
    for line in bytes.split(|b| *b == b'\n') {
        let Ok(text) = std::str::from_utf8(line) else {
            continue;
        };
        let fields: Vec<u32> = text
            .split_whitespace()
            .filter_map(parse_u32_field)
            .collect();
        if fields.len() < 3 {
            continue;
        }
        rows.push(ProcRow {
            pid: fields[0],
            pgid: fields[1],
            ppid: fields[2],
        });
    }
    rows
}

#[cfg(unix)]
pub(crate) fn host_protected_pids(rows: &[ProcRow]) -> HashSet<u32> {
    let me = std::process::id();
    let my_pgid = rows
        .iter()
        .find(|row| row.pid == me)
        .map_or(me, |row| row.pgid);
    rows.iter()
        .filter(|row| row.pgid == my_pgid)
        .map(|row| row.pid)
        .collect()
}

#[cfg(unix)]
pub(crate) fn is_safe_kill_target(pid: u32, protected: &HashSet<u32>) -> bool {
    pid > INIT_PID && pid != std::process::id() && !protected.contains(&pid)
}

#[cfg(unix)]
pub(crate) fn read_proc_cmdline(pid: u32) -> Option<Vec<u8>> {
    std::fs::read(format!("/proc/{pid}/cmdline")).ok()
}

#[cfg(all(unix, test))]
pub(crate) fn read_proc_environ(pid: u32) -> Option<Vec<u8>> {
    std::fs::read(format!("/proc/{pid}/environ")).ok()
}

#[cfg(unix)]
pub(crate) fn looks_like_agent_acp_cmdline(cmdline: &[u8]) -> bool {
    let args: Vec<&[u8]> = cmdline
        .split(|&b| b == 0)
        .filter(|part| !part.is_empty())
        .collect();
    let Some(last) = args.last() else {
        return false;
    };
    if *last != b"acp" {
        return false;
    }
    args.iter()
        .any(|arg| *arg == b"agent" || arg.ends_with(b"/agent"))
}

#[cfg(unix)]
pub(crate) fn looks_like_malvin_agent_acp(pid: u32) -> bool {
    read_proc_cmdline(pid).is_some_and(|cmdline| looks_like_agent_acp_cmdline(&cmdline))
}

#[cfg(unix)]
pub fn spawned_pids_since_baseline(baseline: &HashSet<u32>) -> HashSet<u32> {
    let rows = list_proc_rows().unwrap_or_default();
    let protected = host_protected_pids(&rows);
    snapshot_pids()
        .into_iter()
        .filter(|pid| !baseline.contains(pid) && is_safe_kill_target(*pid, &protected))
        .collect()
}

/// Liveness via `kill(2)` with signal 0 — no subprocess fork.
///
/// Forking `kill -0` under sandbox memory pressure can fail with `ENOMEM`/`EAGAIN`.
/// The previous `Command` path mapped that I/O error to "dead", so
/// `sandbox_still_alive` went false and the memory watchdog exited exactly when
/// the limit was about to be breached. Direct `kill(2)` cannot fail that way.
///
/// Semantics: success or `EPERM` ⇒ alive; `ESRCH` ⇒ dead; other errno ⇒ treat as
/// alive (fail closed for the watchdog: keep watching rather than abandon).
#[cfg(unix)]
#[allow(unsafe_code)]
pub(crate) fn pid_alive(pid: u32) -> bool {
    let Ok(pid_i) = i32::try_from(pid) else {
        return false;
    };
    let rc = unsafe { libc::kill(pid_i, 0) };
    if rc == 0 {
        return true;
    }
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(unix)]
#[allow(unsafe_code)]
pub(crate) fn signal_pid(pid: u32, signal: i32) {
    let Ok(pid_i) = i32::try_from(pid) else {
        return;
    };
    let _ = unsafe { libc::kill(pid_i, signal) };
}

/// Signal a process group via `kill(2)` with a negative pid (same as `kill -SIGNAL -- -PGID`).
#[cfg(unix)]
#[allow(unsafe_code)]
pub fn signal_process_group(process_group_id: u32, signal: i32) {
    let Ok(pgid) = i32::try_from(process_group_id) else {
        return;
    };
    let _ = unsafe { libc::kill(-pgid, signal) };
}

#[cfg(unix)]
pub(crate) fn process_group_member_pids(pgid: u32) -> HashSet<u32> {
    list_proc_rows()
        .map(|rows| {
            rows.into_iter()
                .filter(|row| row.pgid == pgid)
                .map(|row| row.pid)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(not(unix))]
pub fn snapshot_pids() -> std::collections::HashSet<u32> {
    std::collections::HashSet::new()
}

#[cfg(not(unix))]
pub fn spawned_pids_since_baseline(
    _: &std::collections::HashSet<u32>,
) -> std::collections::HashSet<u32> {
    std::collections::HashSet::new()
}

#[cfg(not(unix))]
pub fn signal_process_group(_: u32, _: i32) {}

#[cfg(all(test, unix))]
#[path = "unix_process_group_ps_tests.rs"]
pub(crate) mod unix_process_group_ps_tests;
