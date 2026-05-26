use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub(in crate::process_group_rss) fn linux_pids_rss_bytes(pids: &HashSet<u32>) -> Option<u64> {
    let mut total = 0u64;
    let mut saw = false;
    for pid in pids {
        let status_path = format!("/proc/{pid}/status");
        let status = match fs::read_to_string(&status_path) {
            Ok(s) => s,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(_) => continue,
        };
        let Some(bytes) = parse_status_vm_rss_bytes(&status) else {
            continue;
        };
        saw = true;
        total = total.saturating_add(bytes);
    }
    saw.then_some(total)
}

pub(in crate::process_group_rss) fn linux_process_group_rss_bytes(pgid: u32) -> Option<u64> {
    let entries = fs::read_dir("/proc").ok()?;
    let mut total = 0u64;
    let mut saw_member = false;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let pid = parse_proc_pid_dir_name(name.to_str()?)?;
        let stat_path = format!("/proc/{pid}/stat");
        let stat = match fs::read_to_string(&stat_path) {
            Ok(s) => s,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(_) => continue,
        };
        let proc_pgid = parse_stat_pgrp(&stat)?;
        if proc_pgid != pgid {
            continue;
        }
        saw_member = true;
        let status_path = format!("/proc/{pid}/status");
        let status = fs::read_to_string(&status_path).ok()?;
        total = total.saturating_add(parse_status_vm_rss_bytes(&status)?);
    }
    saw_member.then_some(total)
}

pub(in crate::process_group_rss) fn parse_proc_pid_dir_name(name: &str) -> Option<u32> {
    if name.chars().all(|c| c.is_ascii_digit()) {
        name.parse().ok()
    } else {
        None
    }
}

pub(in crate::process_group_rss) fn parse_stat_pgrp(stat_line: &str) -> Option<u32> {
    let after_comm = stat_line.rsplit_once(')')?.1.trim_start();
    let mut it = after_comm.split_whitespace();
    it.next()?;
    it.next()?;
    it.next()?.parse().ok()
}

pub(in crate::process_group_rss) fn parse_status_vm_rss_bytes(status: &str) -> Option<u64> {
    for line in status.lines() {
        let rest = line.strip_prefix("VmRSS:")?;
        let kb_str = rest.trim().strip_suffix(" kB")?.trim();
        let kb: u64 = kb_str.parse().ok()?;
        return kb.checked_mul(1024);
    }
    None
}

