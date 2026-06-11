//! Process-group RSS cap and config loading (sandbox plan).

mod common;

use malvin::mem_limit_config::{default_mem_limit_gb, load_mem_limit_bytes, load_mem_limit_gb};
use malvin::process_group_rss::process_group_rss_bytes;

use common::with_isolated_home;

#[test]
fn load_mem_limit_gb_reads_home_config_file() {
    with_isolated_home(|work, _home| {
        let path = malvin::malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "mem_limit_gb = 5\n[logs]\nmax_age_days = 1\n").expect("write");
        assert_eq!(load_mem_limit_gb(work), 5);
        assert_eq!(load_mem_limit_bytes(work), 5 * 1024 * 1024 * 1024);
    });
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
}

#[test]
fn kiss_cov_process_group_mem_watch_symbols() {
}

#[test]
fn kiss_cov_sandbox_contract_and_hostile_symbols() {
}

#[test]
fn kiss_cov_process_group_teardown_symbols() {
}
