#[cfg(unix)]
use std::collections::HashSet;
#[cfg(unix)]
use std::process::Stdio;

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
    list_pids_from_ps().unwrap_or_default()
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
    let out = std::process::Command::new("ps")
        .args(["-ax", "-o", "pid=", "-o", "pgid=", "-o", "ppid="])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    Some(parse_proc_rows(&out.stdout))
}

#[cfg(unix)]
pub(crate) fn parse_pid_list(bytes: &[u8]) -> HashSet<u32> {
    let mut pids = HashSet::new();
    for line in bytes.split(|b| *b == b'\n') {
        if let Ok(text) = std::str::from_utf8(line) {
            if let Ok(pid) = text.trim().parse::<u32>() {
                pids.insert(pid);
            }
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

#[cfg(unix)]
pub(crate) fn read_proc_environ(pid: u32) -> Option<Vec<u8>> {
    std::fs::read(format!("/proc/{pid}/environ")).ok()
}

/// True when argv ends with `acp` and names the Cursor `agent` binary (malvin's ACP child pattern).
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
    args.iter().any(|arg| *arg == b"agent" || arg.ends_with(b"/agent"))
}

#[cfg(unix)]
pub(crate) fn looks_like_malvin_agent_acp(pid: u32) -> bool {
    if read_proc_cmdline(pid).is_some_and(|cmdline| looks_like_agent_acp_cmdline(&cmdline)) {
        return true;
    }
    read_proc_environ(pid).is_some_and(|environ| {
        environ
            .split(|&b| b == 0)
            .any(|entry| entry.starts_with(b"MALVIN_WORKSPACE="))
    })
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

#[cfg(unix)]
pub(crate) fn pid_alive(pid: u32) -> bool {
    let Ok(pid_i) = i32::try_from(pid) else {
        return false;
    };
    std::process::Command::new("kill")
        .args(["-0", &pid_i.to_string()])
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(unix)]
pub(crate) fn signal_pid(pid: u32, signal: i32) {
    let Ok(pid_i) = i32::try_from(pid) else {
        return;
    };
    let signal = format!("-{signal}");
    let _ = std::process::Command::new("kill")
        .arg(signal)
        .arg("--")
        .arg(pid_i.to_string())
        .stderr(Stdio::null())
        .status();
}

#[cfg(unix)]
pub fn signal_process_group(process_group_id: u32, signal: i32) {
    let Ok(pgid) = i32::try_from(process_group_id) else {
        return;
    };
    let target = format!("-{pgid}");
    let signal = format!("-{signal}");
    let _ = std::process::Command::new("kill")
        .arg(signal)
        .arg("--")
        .arg(target)
        .stderr(Stdio::null())
        .status();
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
