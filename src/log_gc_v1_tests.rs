use super::log_gc_prune::{over_count_cap, prune_run_dirs};
use super::*;
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
        max_count: Some(3),
        max_age_days: None,
        max_bytes: None,
    };
    assert!(!over_count_cap(runs.len(), config.max_count));
    let removed = prune_run_dirs(&mut runs, &config, None).removed;
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
        max_count: Some(2),
        max_age_days: None,
        max_bytes: None,
    };
    let removed = prune_run_dirs(&mut runs, &config, None).removed;
    assert_eq!(removed, 1);
    assert!(!logs.join(RUN_OLDEST).exists());
}

#[test]
fn some_zero_count_is_a_real_cap() {
    assert!(!over_count_cap(4, None));
    assert!(over_count_cap(4, Some(0)));
    assert!(!over_count_cap(2, Some(2)));
}

#[test]
fn absent_count_cap_keeps_every_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = crate::workspace_paths::malvin_logs_root(tmp.path());
    for name in [RUN_OLDEST, RUN_MID, RUN_NEWEST] {
        std::fs::create_dir_all(logs.join(name)).expect("mkdir");
    }
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_count: None,
        max_age_days: None,
        max_bytes: None,
    };
    assert!(!over_count_cap(runs.len(), config.max_count));
    let removed = prune_run_dirs(&mut runs, &config, None).removed;
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
        max_count: None,
        max_age_days: None,
        max_bytes: Some(500),
    };
    let removed = prune_run_dirs(&mut runs, &config, None).removed;
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

fn make_read_only_dir_with_file(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(dir).expect("mkdir");
    let file = dir.join("f");
    std::fs::write(&file, b"x").expect("write");
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o500)).expect("chmod");
    if std::fs::write(dir.join("probe"), b"x").is_ok() {
        restore_writable(dir);
        return None;
    }
    Some(file)
}

fn restore_writable(dir: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700)).expect("chmod back");
}

#[test]
fn remove_tree_reports_innermost_undeletable_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ro = tmp.path().join("run").join("ro");
    let Some(file) = make_read_only_dir_with_file(&ro) else {
        return;
    };
    let err = remove_tree(&tmp.path().join("run"));
    restore_writable(&ro);
    let (stuck, _) = err.expect_err("read-only dir must block removal");
    assert_eq!(stuck, file);
}

#[test]
fn undeletable_oldest_run_aborts_and_spares_newer_runs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = tmp.path().join("logs");
    let ro = logs.join("20250101_000000_stuck001").join("ro");
    let Some(file) = make_read_only_dir_with_file(&ro) else {
        return;
    };
    for name in [RUN_OLDEST, RUN_MID, RUN_NEWEST] {
        std::fs::create_dir_all(logs.join(name)).expect("mkdir");
    }
    let mut runs = list_run_dirs(&logs);
    runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let config = LogsGcConfig {
        max_count: None,
        max_age_days: Some(1),
        max_bytes: None,
    };
    crate::output::clear_captured_stderr_lines();
    let tally = prune_run_dirs(&mut runs, &config, None);
    let lines = crate::output::take_captured_stderr_lines();
    restore_writable(&ro);
    assert!(tally.aborted);
    assert_eq!(tally.removed, 0);
    for name in [RUN_OLDEST, RUN_MID, RUN_NEWEST] {
        assert!(logs.join(name).is_dir(), "{name} must survive the abort");
    }
    let abs = std::path::absolute(&file).expect("absolute");
    assert!(
        lines.iter().any(|l| l.contains(&abs.display().to_string())),
        "error must name {}: {lines:?}",
        abs.display()
    );
}
