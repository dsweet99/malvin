//! Sum resident set size for all processes in a Unix process group.

use std::collections::HashSet;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod other;

/// Total RSS in bytes for every process in `pgid`, or `None` if the OS query failed.
#[must_use]
pub fn process_group_rss_bytes(pgid: u32) -> Option<u64> {
    if pgid == 0 {
        return None;
    }
    #[cfg(target_os = "linux")]
    {
        linux::linux_process_group_rss_bytes(pgid)
    }
    #[cfg(target_os = "macos")]
    {
        macos::macos_process_group_rss_bytes(pgid)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        other::other_process_group_rss_bytes(_pgid)
    }
}

/// Sum RSS for each pid in `pids`, or `None` if every query failed.
#[must_use]
pub fn pids_rss_bytes(pids: &HashSet<u32>) -> Option<u64> {
    if pids.is_empty() {
        return Some(0);
    }
    #[cfg(target_os = "linux")]
    {
        linux::linux_pids_rss_bytes(pids)
    }
    #[cfg(target_os = "macos")]
    {
        macos::macos_pids_rss_bytes(pids)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = pids;
        None
    }
}

#[cfg(test)]
#[path = "process_group_rss_tests.rs"]
mod process_group_rss_tests;
