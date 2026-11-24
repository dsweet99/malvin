#[cfg(target_os = "linux")]
use super::linux::{
    linux_pid_rss_bytes, linux_pid_sandbox_bytes, linux_pid_uss_bytes, linux_pids_sandbox_bytes,
    parse_proc_kib_field, parse_smaps_rollup_pss_bytes, parse_smaps_rollup_uss_bytes,
    parse_status_vm_rss_bytes,
};
#[test]
fn pids_sandbox_bytes_includes_current_process_and_pids_sandbox_bytes_empty_is_zero() {
    {
        let mut pids = std::collections::HashSet::new();
        pids.insert(std::process::id());
        let bytes = super::pids_sandbox_bytes(&pids).expect("sandbox bytes");
        assert!(bytes > 0);
    }
    {
        let pids = std::collections::HashSet::new();
        assert_eq!(super::pids_sandbox_bytes(&pids), Some(0));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn parse_status_vm_rss_bytes_converts_kib_to_bytes() {
    let status = "Name:\tsleep\nVmRSS:\t  2048 kB\n";
    assert_eq!(parse_status_vm_rss_bytes(status), Some(2048 * 1024));
}

#[cfg(target_os = "linux")]
#[test]
fn parse_smaps_rollup_pss_bytes_converts_kib_to_bytes() {
    let rollup = "Pss:               1024 kB\n";
    assert_eq!(parse_smaps_rollup_pss_bytes(rollup), Some(1024 * 1024));
}

#[cfg(target_os = "linux")]
#[test]
fn parse_proc_kib_field_reads_prefixed_line() {
    assert_eq!(
        parse_proc_kib_field("Pss:  512 kB\n", "Pss:"),
        Some(512 * 1024)
    );
}

#[cfg(target_os = "linux")]
#[test]
fn parse_smaps_rollup_uss_bytes_sums_private_pages() {
    let rollup = "Private_Clean:          40 kB\nPrivate_Dirty:          88 kB\n";
    assert_eq!(parse_smaps_rollup_uss_bytes(rollup), Some(128 * 1024));
}

#[cfg(target_os = "linux")]
#[test]
fn parse_smaps_rollup_uss_bytes_derives_from_shared_subtraction() {
    let rollup =
        "Rss:                1796 kB\nShared_Clean:       1668 kB\nShared_Dirty:          0 kB\n";
    assert_eq!(parse_smaps_rollup_uss_bytes(rollup), Some(128 * 1024));
}

#[cfg(target_os = "linux")]
#[test]
fn linux_pids_sandbox_bytes_counts_private_allocation() {
    let allocation = vec![0x5a_u8; 8 * 1024 * 1024];
    let mut pids = std::collections::HashSet::new();
    pids.insert(std::process::id());
    let sandbox = linux_pids_sandbox_bytes(&pids).expect("sandbox bytes");
    // Private/shared accounting can change between reads as peer processes exit.
    // Each sample must include the touched allocation while it remains live.
    assert!(sandbox >= allocation.len() as u64, "sandbox={sandbox}");
    if let Some(uss) = linux_pid_uss_bytes(std::process::id()) {
        assert!(uss >= allocation.len() as u64, "uss={uss}");
    }
    std::hint::black_box(&allocation);
    assert_sandbox_bytes_fall_back_to_rss_for_a_process_whose_smaps_rollup_is_unreadable();
}

#[cfg(target_os = "linux")]
fn assert_sandbox_bytes_fall_back_to_rss_for_a_process_whose_smaps_rollup_is_unreadable() {
    // A non-dumpable process hides /proc/PID/smaps_rollup but not /proc/PID/status.
    // SAFETY: the child calls only async-signal-safe functions before _exit.
    let child = unsafe { libc::fork() };
    if child == 0 {
        unsafe {
            libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
            libc::pause();
            libc::_exit(0);
        }
    }
    assert!(child > 0, "fork failed");
    let pid = u32::try_from(child).expect("pid");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while linux_pid_uss_bytes(pid).is_some() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let uss = linux_pid_uss_bytes(pid);
    let rss = linux_pid_rss_bytes(pid);
    let sandbox = linux_pid_sandbox_bytes(pid);
    let own = std::iter::once(std::process::id()).collect::<std::collections::HashSet<_>>();
    let both: std::collections::HashSet<u32> = own.iter().copied().chain([pid]).collect();
    let both_bytes = linux_pids_sandbox_bytes(&both);
    unsafe {
        libc::kill(child, libc::SIGKILL);
        libc::waitpid(child, std::ptr::null_mut(), 0);
    }
    assert_eq!(uss, None, "child should hide smaps_rollup");
    assert!(rss.expect("rss") > 0);
    assert_eq!(sandbox, rss);
    assert!(both_bytes.expect("both") >= rss.expect("rss"));
}
