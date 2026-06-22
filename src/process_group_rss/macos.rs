use std::collections::HashSet;
use std::process::Command;

pub(in crate::process_group_rss) fn macos_pids_rss_bytes(pids: &HashSet<u32>) -> Option<u64> {
    let pid_list: Vec<String> = pids.iter().map(std::string::ToString::to_string).collect();
    let joined = pid_list.join(",");
    let out = Command::new("ps")
        .args(["-o", "rss=", "-p", &joined])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let mut total_kib = 0u64;
    let mut saw_line = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let kb: u64 = trimmed.parse().ok()?;
        saw_line = true;
        total_kib = total_kib.saturating_add(kb);
    }
    saw_line.then(|| total_kib.saturating_mul(1024))
}

pub(in crate::process_group_rss) fn macos_process_group_rss_bytes(pgid: u32) -> Option<u64> {
    let out = Command::new("ps")
        .args(["-g", &pgid.to_string(), "-o", "rss="])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let mut total_kib = 0u64;
    let mut saw_line = false;
    for line in text.lines() {
        let kb: u64 = line.trim().parse().ok()?;
        saw_line = true;
        total_kib = total_kib.saturating_add(kb);
    }
    saw_line.then(|| total_kib.saturating_mul(1024))
}
