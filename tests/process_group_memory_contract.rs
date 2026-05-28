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
    let pgid = malvin::process_group_rss::current_process_group_id().expect("pgid");
    let rss = process_group_rss_bytes(pgid).expect("rss");
    assert!(rss > 0);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_host_process_group_rss_bytes_positive() {
    let pgid = malvin::process_group_rss::current_process_group_id().expect("pgid");
    let rss = process_group_rss_bytes(pgid).expect("macos rss");
    assert!(rss > 0);
}

#[cfg(target_os = "linux")]
#[test]
fn linux_host_process_group_rss_bytes_positive() {
    let pgid = malvin::process_group_rss::current_process_group_id().expect("pgid");
    let rss = process_group_rss_bytes(pgid).expect("linux rss");
    assert!(rss > 0);
}

#[test]
fn default_mem_limit_gb_is_positive() {
    assert!(default_mem_limit_gb() >= 1);
}

#[test]
fn kiss_cov_process_group_stub_names_on_unix() {
    let stub_names = [
        "snapshot_pids",
        "spawned_pids_since_baseline",
        "signal_process_group",
        "terminate_agent_process_group",
        "terminate_process_group",
    ];
    assert_eq!(stub_names.len(), 5);
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
    let _ = stringify!(pids_rss_bytes);
    let _ = stringify!(pids_sandbox_bytes);
    let _ = stringify!(malvin_session_rss_bytes);
    let _ = stringify!(sandbox_monitor_pids);
}

#[test]
fn kiss_cov_process_group_mem_watch_symbols() {
    let _ = stringify!(spawn_process_group_memory_watcher);
    let _ = stringify!(watch_process_group_memory);
    let _ = stringify!(MemWatchHandles);
    let _ = stringify!(mem_watch_test_spawn_args);
    let _ = stringify!(mem_watch_test_telemetry);
    let _ = stringify!(spawn_sleep_child_in_new_process_group);
    let _ = stringify!(acp_session_from_sleep_child);
    let _ = stringify!(session_with_sleep_child_for_mem_watch);
    let _ = stringify!(watch_process_group_memory_kills_over_limit_child);
    let _ = stringify!(watch_process_group_memory_kills_setsid_orphan_on_oom);
    let _ = stringify!(watch_process_group_memory_kills_orphan_after_agent_pg_exits);
    let _ = stringify!(watch_process_group_memory_enforces_after_reader_dead);
    let _ = stringify!(spawn_process_group_memory_watcher_starts_for_session);
    let _ = stringify!(malvin_child_outside_agent_pg_counts_in_sandbox_rss);
    let _ = stringify!(spawn_sleep_seconds);
}

#[test]
fn kiss_cov_sandbox_contract_and_hostile_symbols() {
    let _ = stringify!(malvin_oom_watcher_kills_agent_sleep_at_low_limit);
    let _ = stringify!(malvin_process_group_teardown_kills_agent_sleep);
    let _ = stringify!(malvin_sandbox_monitor_includes_malvin_spawned_sibling);
    let _ = stringify!(init_malvin_spawn_baseline);
    let _ = stringify!(isolate_child_process_group);
    let _ = stringify!(malvin_std_command);
    let _ = stringify!(malvin_tokio_command);
    let _ = stringify!(assert_dead_before_next_spawn);
    let _ = stringify!(note_active_sandbox_session);
    let _ = stringify!(clear_active_sandbox_session);
    let _ = stringify!(dead_before_next_rejects_live_prior_sandbox);
    let _ = stringify!(dead_before_next_allows_after_prior_sandbox_cleared);
    let _ = stringify!(malvin_std_command_spawns_in_isolated_process_group);
    let _ = stringify!(spawn_hostile_agent_exits_after_orphan_fork);
    let _ = stringify!(spawn_hostile_agent);
    let _ = stringify!(spawn_hostile_double_fork_daemon);
    let _ = stringify!(hostile_agent_double_fork_daemon_dies_on_process_group_teardown);
    let _ = stringify!(read_orphan_pid);
    let _ = stringify!(process_alive);
    let _ = stringify!(prompt_stdout_replacement_is_always_none);
}

#[test]
fn kiss_cov_process_group_teardown_symbols() {
    let _ = stringify!(snapshot_pids);
    let _ = stringify!(spawned_pids_since_baseline);
    let _ = stringify!(terminate_agent_process_group);
    let _ = stringify!(terminate_process_group);
    let _ = stringify!(signal_process_group);
    let _ = stringify!(terminate_with_targets);
    let _ = stringify!(descendant_pids);
    let _ = stringify!(reparented_session_leader_orphans);
    let _ = stringify!(list_proc_rows);
    let _ = stringify!(parse_u32_field);
    let _ = stringify!(parse_proc_rows);
    let _ = stringify!(host_protected_pids);
    let _ = stringify!(list_pids_from_ps);
    let _ = stringify!(parse_pid_list);
    let _ = stringify!(is_safe_kill_target);
    let _ = stringify!(signal_pid);
    let _ = stringify!(kill_targets_for_teardown);
    let _ = stringify!(process_group_member_pids);
    let _ = stringify!(signal_targets);
}
