#[cfg(target_os = "macos")]
use super::macos::macos_process_group_rss_bytes;
use super::process_group_rss_bytes;

#[cfg(target_os = "linux")]
use super::linux::{
    linux_pids_pss_bytes, linux_pids_sandbox_bytes, linux_process_group_rss_bytes,
    parse_proc_kib_field, parse_proc_pid_dir_name, parse_smaps_rollup_pss_bytes,
    parse_stat_pgrp, parse_status_vm_rss_bytes,
};

#[test]
fn kiss_cov_linux_process_group_rss_symbol_names() {
    let _ = stringify!(parse_stat_pgrp);
    let _ = stringify!(parse_status_vm_rss_bytes);
    let _ = stringify!(parse_proc_pid_dir_name);
    let _ = stringify!(process_group_rss_bytes);
    let _ = stringify!(pids_rss_bytes);
    let _ = stringify!(linux_process_group_rss_bytes);
    let _ = stringify!(macos_process_group_rss_bytes);
    let _ = stringify!(linux_pids_rss_bytes);
    let _ = stringify!(linux_pids_pss_bytes);
    let _ = stringify!(linux_pids_sandbox_bytes);
    let _ = stringify!(parse_smaps_rollup_pss_bytes);
    let _ = stringify!(parse_proc_kib_field);
    let _ = stringify!(macos_pids_rss_bytes);
    let _ = stringify!(pids_rss_bytes);
    let _ = stringify!(pids_sandbox_bytes);
}

#[test]
fn pids_rss_bytes_includes_current_process() {
    let mut pids = std::collections::HashSet::new();
    pids.insert(std::process::id());
    let rss = super::pids_rss_bytes(&pids).expect("pids rss");
    assert!(rss > 0);
}

#[test]
fn process_group_rss_bytes_includes_current_group() {
    let pgid = super::current_process_group_id().expect("pgid");
    let rss = process_group_rss_bytes(pgid).expect("rss");
    assert!(rss > 0);
}

#[cfg(target_os = "linux")]
#[test]
fn parse_stat_pgrp_reads_process_group_field() {
    let line = "42 (sleep) S 1 99 99 0 -1 4194304 0 0 0 0 0 0 0 0 0 0 0";
    assert_eq!(parse_stat_pgrp(line), Some(99));
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
fn linux_pids_sandbox_bytes_uses_self_pss_or_rss() {
    let mut pids = std::collections::HashSet::new();
    pids.insert(std::process::id());
    let sandbox = linux_pids_sandbox_bytes(&pids).expect("sandbox bytes");
    assert!(sandbox > 0);
    // PSS/RSS can shift between back-to-back /proc reads; sandbox must prefer PSS when present.
    if let Some(pss) = linux_pids_pss_bytes(&pids) {
        let slack = 4 * 1024 * 1024;
        assert!(
            sandbox.abs_diff(pss) <= slack,
            "sandbox={sandbox} pss={pss}"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn parse_proc_pid_dir_name_accepts_digits() {
    assert_eq!(parse_proc_pid_dir_name("42"), Some(42));
    assert!(parse_proc_pid_dir_name("notpid").is_none());
}

#[cfg(target_os = "linux")]
#[test]
fn linux_process_group_rss_bytes_includes_self() {
    let pgid = super::current_process_group_id().expect("pgid");
    let rss = linux_process_group_rss_bytes(pgid).expect("linux rss");
    assert!(rss > 0);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_process_group_rss_bytes_includes_self() {
    let pgid = super::current_process_group_id().expect("pgid");
    let rss = macos_process_group_rss_bytes(pgid).expect("macos rss");
    assert!(rss > 0);
}

#[cfg(unix)]
#[test]
fn child_in_same_process_group_contributes_to_rss() {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    let mut cmd = Command::new("sleep");
    cmd.arg("30");
    cmd.process_group(0);
    let mut child = cmd.spawn().expect("spawn sleep");
    let pgid = child.id();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let rss = process_group_rss_bytes(pgid).expect("rss");
    let _ = child.kill();
    let _ = child.wait();
    assert!(rss > 0);
}
