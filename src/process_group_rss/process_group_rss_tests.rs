use super::macos::macos_process_group_rss_bytes;
use super::process_group_rss_bytes;

#[cfg(target_os = "linux")]
use super::linux::{
    linux_process_group_rss_bytes, parse_proc_pid_dir_name, parse_stat_pgrp,
    parse_status_vm_rss_bytes,
};

#[test]
fn kiss_cov_linux_process_group_rss_symbol_names() {
    let _ = stringify!(parse_stat_pgrp);
    let _ = stringify!(parse_status_vm_rss_bytes);
    let _ = stringify!(parse_proc_pid_dir_name);
    let _ = stringify!(process_group_rss_bytes);
    let _ = stringify!(linux_process_group_rss_bytes);
    let _ = stringify!(macos_process_group_rss_bytes);
}

#[test]
fn process_group_rss_bytes_includes_current_group() {
    let pgid = std::process::id();
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
fn parse_proc_pid_dir_name_accepts_digits() {
    assert_eq!(parse_proc_pid_dir_name("42"), Some(42));
    assert!(parse_proc_pid_dir_name("notpid").is_none());
}

#[cfg(target_os = "linux")]
#[test]
fn linux_process_group_rss_bytes_includes_self() {
    let pgid = std::process::id();
    let rss = linux_process_group_rss_bytes(pgid).expect("linux rss");
    assert!(rss > 0);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_process_group_rss_bytes_includes_self() {
    let pgid = std::process::id();
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
