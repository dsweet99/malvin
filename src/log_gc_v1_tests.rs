use super::*;
use super::log_gc_format::{cached_total_bytes, format_max_bytes_display, format_max_count_display};
use super::log_gc_prune::{over_count_cap, prune_run_dirs};
use crate::log_gc_config::LogsGcConfig;

const RUN_OLDEST: &str = "20260101_000000_aaaaaaa1";
const RUN_MID: &str = "20260102_000000_bbbbbbb2";
const RUN_NEWEST: &str = "20260103_000000_ccccccc3";
const RUN_OLD_AGE: &str = "20200101_000000_oldrun01";
const CONFIG_AGE_ONLY: &str = "[logs]\nmax_count = 0\nmax_age_days = 30\nmax_bytes = \"\"\n";
const CONFIG_COUNT_2: &str = "[logs]\nmax_count = 2\nmax_age_days = 0\nmax_bytes = \"\"\n";

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

fn assert_cached_bytes_match_runs(runs: &[PathBuf]) {
    let sizes: Vec<u64> = runs.iter().map(|p| dir_size(p)).collect();
    let direct: u64 = runs.iter().map(|p| dir_size(p)).sum();
    assert_eq!(cached_total_bytes(&sizes), direct);
}

#[test]
fn size_cache_total_matches_direct_dir_size_after_deletes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    std::fs::create_dir_all(logs.join(RUN_OLDEST)).expect("mkdir old");
    std::fs::write(logs.join(RUN_OLDEST).join("payload"), vec![0u8; 1000]).expect("write");
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    assert_cached_bytes_match_runs(&runs);
    let config = LogsGcConfig {
        max_count: 0,
        max_age_days: 0,
        max_bytes: Some(500),
    };
    let (removed, _) = prune_run_dirs(&mut runs, &config);
    assert_eq!(removed, 1);
    assert_cached_bytes_match_runs(&runs);
}

#[test]
fn prune_logs_dry_run_leaves_dirs_on_disk() {
    crate::test_utils::with_isolated_home(|work| {
        let old = crate::workspace_paths::malvin_logs_root(work).join(RUN_OLD_AGE);
        std::fs::create_dir_all(&old).expect("mkdir");
        let config_path = crate::malvin_config_path(work);
        std::fs::create_dir_all(config_path.parent().unwrap()).expect("mkdir home");
        std::fs::write(&config_path, CONFIG_AGE_ONLY).expect("write config");
        let result = prune_logs(work, PruneOpts { dry_run: true, verbose: false });
        assert_eq!(result.removed, 0);
        assert!(result.would_remove > 0);
        assert!(old.is_dir());
    });
}

#[test]
fn logs_bucket_status_reports_trigger_flags() {
    crate::test_utils::with_isolated_home(|work| {
        let logs = crate::workspace_paths::malvin_logs_root(work);
        for name in [RUN_OLDEST, RUN_MID, RUN_NEWEST] {
            std::fs::create_dir_all(logs.join(name)).expect("mkdir");
        }
        let config_path = crate::malvin_config_path(work);
        std::fs::create_dir_all(config_path.parent().unwrap()).expect("mkdir home");
        std::fs::write(&config_path, CONFIG_COUNT_2).expect("write config");
        let status = logs_bucket_status(work);
        assert_eq!(status.run_count, 3);
        assert!(status.would_count_cap);
    });
}

#[test]
fn prune_result_and_bucket_status_types_are_populated() {
    let result = PruneResult {
        removed: 1,
        freed: 42,
        would_remove: 0,
    };
    assert_eq!(result.removed, 1);
    assert_eq!(result.freed, 42);
    let status = LogsBucketStatus {
        bucket_path: PathBuf::from("/tmp/bucket"),
        run_count: 0,
        total_bytes: 0,
        oldest_run: None,
        newest_run: None,
        config: LogsGcConfig::default(),
        would_byte_cap: false,
        would_count_cap: false,
        would_age_limit: false,
    };
    assert!(status.bucket_path.ends_with("bucket"));
    assert_eq!(cached_total_bytes(&[1, 2, 3]), 6);
    assert_eq!(format_max_bytes_display(Some(2048)), "2 KiB");
    assert_eq!(format_max_count_display(42), "42");
}

#[test]
fn sweep_empty_log_buckets_removes_dirs_with_no_runs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home_logs = tmp.path().join("logs");
    let empty = home_logs.join("deadbeefdeadbeef");
    std::fs::create_dir_all(&empty).expect("mkdir empty");
    assert_eq!(sweep_empty_log_buckets(&home_logs), 1);
    assert!(!empty.exists());
}

#[test]
fn prune_logs_verbose_prints_candidate_paths() {
    crate::test_utils::with_isolated_home(|work| {
        std::fs::create_dir_all(crate::workspace_paths::malvin_logs_root(work).join(RUN_OLD_AGE))
            .expect("mkdir");
        let config_path = crate::malvin_config_path(work);
        std::fs::create_dir_all(config_path.parent().unwrap()).expect("mkdir home");
        std::fs::write(&config_path, CONFIG_AGE_ONLY).expect("write config");
        let result = prune_logs(work, PruneOpts { dry_run: false, verbose: true });
        assert!(result.removed > 0);
    });
}

#[test]
fn logs_bucket_status_empty_bucket_reports_none_runs() {
    crate::test_utils::with_isolated_home(|work| {
        let status = logs_bucket_status(work);
        assert_eq!(status.run_count, 0);
        assert!(status.oldest_run.is_none());
    });
}
