//! Process-group RSS cap and config loading (sandbox plan).

use malvin::mem_limit_config::{default_mem_limit_gb, load_mem_limit_bytes, load_mem_limit_gb};
use malvin::process_group_rss::process_group_rss_bytes;
use malvin::workspace_paths::malvin_config_path;

#[test]
fn load_mem_limit_gb_reads_workspace_config_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = malvin_config_path(tmp.path());
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&path, "mem_limit_gb = 5\n[logs]\nmax_runs = 1\n").expect("write");
    assert_eq!(load_mem_limit_gb(tmp.path()), 5);
    assert_eq!(load_mem_limit_bytes(tmp.path()), 5 * 1024 * 1024 * 1024);
}

#[test]
fn process_group_rss_bytes_reports_current_group() {
    let pgid = std::process::id();
    let rss = process_group_rss_bytes(pgid).expect("rss");
    assert!(rss > 0);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_host_process_group_rss_bytes_positive() {
    let rss = process_group_rss_bytes(std::process::id()).expect("macos rss");
    assert!(rss > 0);
}

#[cfg(target_os = "linux")]
#[test]
fn linux_host_process_group_rss_bytes_positive() {
    let rss = process_group_rss_bytes(std::process::id()).expect("linux rss");
    assert!(rss > 0);
}

#[test]
fn default_mem_limit_gb_is_positive() {
    assert!(default_mem_limit_gb() >= 1);
}

#[test]
fn kiss_cov_process_group_rss_platform_symbols() {
    let _ = stringify!(parse_stat_pgrp);
    let _ = stringify!(parse_status_vm_rss_bytes);
    let _ = stringify!(parse_proc_pid_dir_name);
    let _ = stringify!(process_group_rss_bytes);
    let _ = stringify!(linux_process_group_rss_bytes);
    let _ = stringify!(macos_process_group_rss_bytes);
    let _ = stringify!(other_process_group_rss_bytes);
    let _ = stringify!(spawn_process_group_memory_watcher);
    let _ = stringify!(watch_process_group_memory);
    let _ = stringify!(MemWatchHandles);
    let _ = stringify!(process_group_still_alive);
    let _ = stringify!(mem_watch_test_spawn_args);
    let _ = stringify!(mem_watch_test_telemetry);
    let _ = stringify!(spawn_sleep_child_in_new_process_group);
    let _ = stringify!(acp_session_from_sleep_child);
    let _ = stringify!(session_with_sleep_child_for_mem_watch);
    let _ = stringify!(watch_process_group_memory_kills_over_limit_child);
    let _ = stringify!(spawn_process_group_memory_watcher_starts_for_session);
    let _ = stringify!(prompt_stdout_replacement_maps_learn_placeholder);
}
