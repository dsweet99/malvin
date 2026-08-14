#[cfg(target_os = "linux")]
use super::linux::{
    linux_pids_sandbox_bytes, linux_pids_uss_bytes, parse_proc_kib_field,
    parse_smaps_rollup_pss_bytes, parse_smaps_rollup_uss_bytes, parse_status_vm_rss_bytes,
};

#[test]
fn pids_sandbox_bytes_includes_current_process() {
    let mut pids = std::collections::HashSet::new();
    pids.insert(std::process::id());
    let bytes = super::pids_sandbox_bytes(&pids).expect("sandbox bytes");
    assert!(bytes > 0);
}

#[test]
fn pids_sandbox_bytes_empty_is_zero() {
    let pids = std::collections::HashSet::new();
    assert_eq!(super::pids_sandbox_bytes(&pids), Some(0));
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
    assert_eq!(parse_proc_kib_field("Pss:  512 kB\n", "Pss:"), Some(512 * 1024));
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
    let rollup = "Rss:                1796 kB\nShared_Clean:       1668 kB\nShared_Dirty:          0 kB\n";
    assert_eq!(parse_smaps_rollup_uss_bytes(rollup), Some(128 * 1024));
}

#[cfg(target_os = "linux")]
#[test]
fn linux_pids_sandbox_bytes_uses_self_uss_or_rss() {
    let mut pids = std::collections::HashSet::new();
    pids.insert(std::process::id());
    let sandbox = linux_pids_sandbox_bytes(&pids).expect("sandbox bytes");
    assert!(sandbox > 0);
    if let Some(uss) = linux_pids_uss_bytes(&pids) {
        let slack = 4 * 1024 * 1024;
        assert!(
            sandbox.abs_diff(uss) <= slack,
            "sandbox={sandbox} uss={uss}"
        );
    }
}
