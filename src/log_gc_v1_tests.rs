use super::*;
use super::log_gc_prune::{over_count_cap, prune_run_dirs};
use crate::log_gc_config::LogsGcConfig;

const RUN_OLDEST: &str = "20260101_000000_aaaaaaa1";
const RUN_MID: &str = "20260102_000000_bbbbbbb2";
const RUN_NEWEST: &str = "20260103_000000_ccccccc3";

#[test]
fn over_count_cap_at_limit_does_not_prune() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    for name in [RUN_OLDEST, RUN_MID, RUN_NEWEST] {
        std::fs::create_dir_all(logs.join(name)).expect("mkdir");
    }
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_count: 3,
        max_age_days: 0,
        max_bytes: None,
    };
    assert!(!over_count_cap(runs.len(), config.max_count));
    let (removed, _) = prune_run_dirs(&mut runs, &config);
    assert_eq!(removed, 0);
    assert_eq!(runs.len(), 3);
}

#[test]
fn prune_removes_oldest_when_over_count_cap() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    for name in [RUN_OLDEST, RUN_MID, RUN_NEWEST] {
        std::fs::create_dir_all(logs.join(name)).expect("mkdir");
    }
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_count: 2,
        max_age_days: 0,
        max_bytes: None,
    };
    let (removed, _) = prune_run_dirs(&mut runs, &config);
    assert_eq!(removed, 1);
    assert!(!logs.join(RUN_OLDEST).exists());
}

#[test]
fn max_count_zero_means_unlimited_count() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    for name in [RUN_OLDEST, RUN_MID, RUN_NEWEST] {
        std::fs::create_dir_all(logs.join(name)).expect("mkdir");
    }
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_count: 0,
        max_age_days: 0,
        max_bytes: None,
    };
    assert!(!over_count_cap(runs.len(), config.max_count));
    let (removed, _) = prune_run_dirs(&mut runs, &config);
    assert_eq!(removed, 0);
}

#[test]
fn size_total_matches_direct_dir_size_after_deletes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    let run = logs.join(RUN_OLDEST);
    std::fs::create_dir_all(&run).expect("mkdir old");
    std::fs::write(run.join("payload"), vec![0u8; 1000]).expect("write");
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_count: 0,
        max_age_days: 0,
        max_bytes: Some(500),
    };
    let (removed, _) = prune_run_dirs(&mut runs, &config);
    assert_eq!(removed, 1);
    assert!(runs.is_empty());
}

#[test]
fn prune_result_type_is_populated() {
    let result = PruneResult {
        removed: 1,
        freed: 42,
    };
    assert_eq!(result.removed, 1);
    assert_eq!(result.freed, 42);
}
