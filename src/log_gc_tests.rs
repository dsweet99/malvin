use super::*;
use crate::log_gc_config::{
    load_logs_gc_config, parse_byte_size, parse_logs_gc_config, parse_max_bytes_value, read_u64,
    split_byte_size, LogsGcConfig,
};

const RUN_OLDEST: &str = "20260101_000000_aaaaaaa1";
const RUN_MID: &str = "20260102_000000_bbbbbbb2";
const RUN_NEWEST: &str = "20260103_000000_ccccccc3";
const RUN_OLD_AGE: &str = "20200101_000000_oldrun01";

#[test]
fn parse_byte_size_accepts_binary_units() {
    assert_eq!(parse_byte_size("2GiB"), Some(2 * 1024_u64.pow(3)));
    assert_eq!(parse_byte_size("512MiB"), Some(512 * 1024_u64.pow(2)));
    assert_eq!(parse_byte_size("1KiB"), Some(1024));
    assert_eq!(parse_byte_size("100B"), Some(100));
}

#[test]
fn parse_byte_size_rejects_invalid() {
    assert!(parse_byte_size("").is_none());
    assert!(parse_byte_size("nope").is_none());
}

#[test]
fn run_dir_timestamp_parses_dirnames() {
    let ts = run_dir_timestamp("20260524_173353_kmdb83bt").expect("ts");
    assert_eq!(ts.format("%Y%m%d_%H%M%S").to_string(), "20260524_173353");
}

#[test]
fn is_run_log_dir_name_matches_malvin_run_dirs() {
    assert!(is_run_log_dir_name("20260524_173353_kmdb83bt"));
    assert!(is_run_log_dir_name(RUN_NEWEST));
    assert!(!is_run_log_dir_name("hand_notes"));
    assert!(!is_run_log_dir_name("20260103_000000"));
    assert!(!is_run_log_dir_name("20260103_000000_ccc"));
}

#[test]
fn load_logs_gc_config_uses_defaults_when_missing() {
    crate::test_utils::with_isolated_home(|work| {
        let cfg = load_logs_gc_config(work);
        assert_eq!(cfg, LogsGcConfig::default());
    });
}

#[test]
fn parse_logs_gc_config_reads_toml() {
    let cfg = parse_logs_gc_config("[logs]\nmax_age_days = 7\nmax_bytes = \"1MiB\"\n").expect("parse");
    assert_eq!(cfg.max_age_days, 7);
    assert_eq!(cfg.max_bytes, parse_byte_size("1MiB"));
}

#[test]
fn log_gc_helpers_cover_policy_edges() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let old = tmp.path().join(RUN_OLD_AGE);
    std::fs::create_dir_all(&old).expect("mkdir");
    std::fs::write(old.join("nested.log"), "x").expect("write");
    let runs = vec![old.clone()];
    let config = LogsGcConfig {
        max_age_days: 30,
        max_bytes: Some(0),
    };
    assert!(over_byte_cap(&runs, Some(0)));
    assert!(!over_byte_cap(&runs, None));
    assert!(over_age_limit(runs.last(), config.max_age_days));
    assert!(needs_prune(&runs, &config));
    assert_eq!(dir_size(&old), dir_size_inner(&old).expect("dir_size_inner"));
    assert!(mtime_as_utc(&old).is_some());
    assert_eq!(read_u64(Some(&toml::Value::Integer(5))), Some(5));
    assert_eq!(
        parse_max_bytes_value(&toml::Value::String(String::new())).expect("empty"),
        None
    );
    assert!(split_byte_size("1GiB").is_some());
    crate::test_utils::with_isolated_home(|work| {
        prune_logs_before_run(work);
    });
}

#[test]
fn parse_logs_gc_config_warns_on_invalid_max_bytes() {
    let err = parse_logs_gc_config("[logs]\nmax_bytes = \"bad\"\n").unwrap_err();
    assert!(err.contains("max_bytes"));
}

#[test]
fn prune_keeps_dated_run_when_arbitrary_subdir_would_sort_newer() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    std::fs::create_dir_all(logs.join("hand_notes")).expect("mkdir");
    std::fs::create_dir_all(logs.join(RUN_NEWEST)).expect("run dir");
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_age_days: 0,
        max_bytes: None,
    };
    prune_run_dirs(&mut runs, &config);
    assert!(
        logs.join(RUN_NEWEST).is_dir(),
        "GC must not remove dated run dirs in favor of arbitrary log subdirs"
    );
    assert!(logs.join("hand_notes").is_dir());
}

#[test]
fn prune_leaves_non_run_log_subdirs_untouched() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    std::fs::create_dir_all(logs.join("hand_notes")).expect("mkdir");
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_age_days: 0,
        max_bytes: None,
    };
    prune_run_dirs(&mut runs, &config);
    assert!(logs.join("hand_notes").is_dir());
}

#[test]
fn prune_removes_run_dir_when_over_age_limit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    let old = logs.join(RUN_OLD_AGE);
    std::fs::create_dir_all(&old).expect("mkdir");
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_age_days: 30,
        max_bytes: None,
    };
    let (removed, _) = prune_run_dirs(&mut runs, &config);
    assert_eq!(removed, 1);
    assert!(!old.exists());
}

fn two_run_dirs_with_payload(logs: &std::path::Path, bytes_each: usize) -> (PathBuf, PathBuf) {
    let old = logs.join(RUN_OLDEST);
    let new = logs.join(RUN_NEWEST);
    std::fs::create_dir_all(&old).expect("mkdir old");
    std::fs::create_dir_all(&new).expect("mkdir new");
    let payload = vec![0u8; bytes_each];
    std::fs::write(old.join("payload"), &payload).expect("write old");
    std::fs::write(new.join("payload"), &payload).expect("write new");
    (old, new)
}

#[test]
fn prune_removes_oldest_when_over_byte_cap() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    let (old, new) = two_run_dirs_with_payload(&logs, 2000);
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_age_days: 0,
        max_bytes: Some(3000),
    };
    let (removed, _) = prune_run_dirs(&mut runs, &config);
    assert_eq!(removed, 1);
    assert!(!old.exists());
    assert!(new.is_dir());
}

#[cfg(unix)]
fn undeletable_oldest_run_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    std::fs::create_dir_all(&logs).expect("mkdir");
    let oldest = logs.join(RUN_OLDEST);
    for name in [RUN_OLDEST, RUN_MID, RUN_NEWEST] {
        std::fs::create_dir_all(logs.join(name)).expect("run dir");
        std::fs::write(logs.join(name).join("payload"), vec![0u8; 600]).expect("write");
    }
    std::fs::set_permissions(&oldest, std::fs::Permissions::from_mode(0o000)).expect("chmod");
    (tmp, logs, oldest)
}

#[cfg(unix)]
#[test]
fn prune_retries_or_reports_when_delete_fails_and_limits_still_exceeded() {
    use std::os::unix::fs::PermissionsExt;

    let (_tmp, logs, oldest) = undeletable_oldest_run_fixture();
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_age_days: 0,
        max_bytes: Some(1000),
    };
    let (removed, _) = prune_run_dirs(&mut runs, &config);

    std::fs::set_permissions(&oldest, std::fs::Permissions::from_mode(0o700)).expect("restore");
    assert_eq!(
        list_run_dirs(&logs).len(),
        2,
        "after a failed delete, GC must still enforce byte cap on disk (got {removed} removed)"
    );
    assert!(
        oldest.is_dir(),
        "undeletable oldest run must not be dropped from enforcement"
    );
}
