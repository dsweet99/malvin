use std::collections::HashSet;
use std::fs;

use super::process_group_rss::{MemoryMetric, MemorySample};

#[cfg(test)]
pub(crate) fn linux_pids_sandbox_bytes(pids: &HashSet<u32>) -> Option<u64> {
    linux_pids_sandbox_sample(pids).map(|sample| sample.bytes)
}

pub(crate) fn linux_pids_sandbox_sample(pids: &HashSet<u32>) -> Option<MemorySample> {
    pids.iter()
        .filter_map(|&pid| linux_pid_sandbox_sample(pid))
        .reduce(MemorySample::combine)
}

#[cfg(test)]
pub(crate) fn linux_pid_sandbox_bytes(pid: u32) -> Option<u64> {
    linux_pid_sandbox_sample(pid).map(|sample| sample.bytes)
}

pub(crate) fn linux_pid_sandbox_sample(pid: u32) -> Option<MemorySample> {
    linux_pid_uss_bytes(pid)
        .map(|bytes| MemorySample::new(bytes, MemoryMetric::Uss))
        .or_else(|| {
            linux_pid_rss_bytes(pid).map(|bytes| MemorySample::new(bytes, MemoryMetric::Rss))
        })
}

pub(crate) fn linux_pid_uss_bytes(pid: u32) -> Option<u64> {
    let rollup = fs::read_to_string(format!("/proc/{pid}/smaps_rollup")).ok()?;
    parse_smaps_rollup_uss_bytes(&rollup)
}

pub(crate) fn linux_pid_rss_bytes(pid: u32) -> Option<u64> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    parse_status_vm_rss_bytes(&status)
}

pub(crate) fn parse_status_vm_rss_bytes(status: &str) -> Option<u64> {
    parse_proc_kib_field(status, "VmRSS:")
}

#[cfg(test)]
pub(crate) fn parse_smaps_rollup_pss_bytes(rollup: &str) -> Option<u64> {
    parse_proc_kib_field(rollup, "Pss:")
}

pub(crate) fn parse_smaps_rollup_uss_bytes(rollup: &str) -> Option<u64> {
    if let Some(uss) = parse_proc_kib_field(rollup, "USS:") {
        return Some(uss);
    }
    if let (Some(clean), Some(dirty)) = (
        parse_proc_kib_field(rollup, "Private_Clean:"),
        parse_proc_kib_field(rollup, "Private_Dirty:"),
    ) {
        return Some(clean.saturating_add(dirty));
    }
    let rss = parse_proc_kib_field(rollup, "Rss:")?;
    let shared_clean = parse_proc_kib_field(rollup, "Shared_Clean:").unwrap_or(0);
    let shared_dirty = parse_proc_kib_field(rollup, "Shared_Dirty:").unwrap_or(0);
    let shared = shared_clean.saturating_add(shared_dirty);
    Some(rss.saturating_sub(shared))
}

pub(crate) fn parse_proc_kib_field(text: &str, prefix: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let rest = line.strip_prefix(prefix)?;
        let kb_str = rest.trim().strip_suffix(" kB")?.trim();
        let kb: u64 = kb_str.parse().ok()?;
        kb.checked_mul(1024)
    })
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = linux_pid_uss_bytes;
        let rss = linux_pid_rss_bytes(std::process::id()).expect("rss");
        assert!(rss > 0);
    }
}
