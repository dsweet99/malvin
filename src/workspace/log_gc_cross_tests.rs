use super::log_gc_prune::prune_run_dirs;
use super::*;
use crate::config::log_gc_config::LogsGcConfig;
use crate::workspace::workspace_paths::{
    malvin_home_logs_root, malvin_logs_root, write_work_dir_manifest,
};

const RUN_OLD_AGE: &str = "20200101_000000_oldrun01";
const RUN_OLDEST: &str = "20260101_000000_aaaaaaa1";
const RUN_NEWEST: &str = "20260103_000000_ccccccc3";
const OTHER_HASH: &str = "deadbeefdeadbeef";
const ORPHAN_HASH: &str = "aaaaaaaaaaaaaaaa";
const AGE_ONLY_TOML: &str = "[logs]\nmax_count = 0\nmax_age_days = 30\nmax_bytes = \"\"\n";

#[test]
fn prune_logs_after_run_created_applies_retention_across_hash_buckets() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let config_path = crate::malvin_config_path(work);
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir config parent");
        }
        std::fs::write(&config_path, AGE_ONLY_TOML).expect("write config");
        let other = malvin_home_logs_root().join(OTHER_HASH);
        let old = other.join(RUN_OLD_AGE);
        std::fs::create_dir_all(&old).expect("seed other hash run");
        std::fs::write(old.join("marker"), "x").expect("marker");
        write_work_dir_manifest(&old, work).expect("manifest");

        let protect = work.join("__no_active_run__");
        prune_logs_after_run_created(work, &protect);

        assert!(!old.exists(), "aged run in foreign hash must be pruned");
        assert!(
            !other.exists(),
            "empty foreign hash bucket must be removed after prune"
        );
    });
}

#[test]
fn prune_logs_after_run_created_removes_empty_orphan_hash_buckets() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let home_logs = malvin_home_logs_root();
        let orphan = home_logs.join(ORPHAN_HASH);
        std::fs::create_dir_all(&orphan).expect("orphan bucket");
        let current = malvin_logs_root(work);
        std::fs::create_dir_all(&current).expect("current bucket");

        let protect = work.join("__no_active_run__");
        prune_logs_after_run_created(work, &protect);

        assert!(!orphan.exists(), "empty orphan hash must be collected");
        assert!(
            current.is_dir(),
            "current workspace hash bucket must be kept even if empty"
        );
    });
}

const NO_LIMITS: LogsGcConfig = LogsGcConfig {
    max_count: None,
    max_age_days: None,
    max_bytes: None,
};
const MISSING_PROJECT_OUTSIDE_TEMP: &str = "/nonexistent-malvin-gc-test/project";

fn seed_run_for_work_dir(
    home_logs: &std::path::Path,
    run: &str,
    work: &std::path::Path,
) -> PathBuf {
    let run_dir = home_logs.join(OTHER_HASH).join(run);
    std::fs::create_dir_all(&run_dir).expect("seed run");
    write_work_dir_manifest(&run_dir, work).expect("manifest");
    run_dir
}

#[test]
fn logs_of_a_missing_project_outside_temp_are_kept_without_a_retention_limit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home_logs = tmp.path().join("logs");
    let work = std::path::Path::new(MISSING_PROJECT_OUTSIDE_TEMP);
    let run = seed_run_for_work_dir(&home_logs, RUN_NEWEST, work);

    let (removed, _) =
        super::log_gc_buckets::prune_all_log_buckets(&home_logs, &NO_LIMITS, None, None);

    assert_eq!(removed, 0);
    assert!(run.is_dir(), "an unmounted project must keep its logs");
}

#[test]
fn logs_of_a_missing_project_outside_temp_still_age_out() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home_logs = tmp.path().join("logs");
    let work = std::path::Path::new(MISSING_PROJECT_OUTSIDE_TEMP);
    let old = seed_run_for_work_dir(&home_logs, RUN_OLD_AGE, work);
    let recent_name = format!("{}_recent01", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let recent = seed_run_for_work_dir(&home_logs, &recent_name, work);
    let config = LogsGcConfig {
        max_age_days: Some(30),
        ..NO_LIMITS
    };

    let (removed, _) =
        super::log_gc_buckets::prune_all_log_buckets(&home_logs, &config, None, None);

    assert_eq!(removed, 1);
    assert!(!old.exists(), "the over-age run must go");
    assert!(recent.is_dir(), "the recent run must stay");
}

#[test]
fn logs_are_kept_when_a_temp_work_dir_cannot_be_checked() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().expect("tempdir");
    let home_logs = tmp.path().join("logs");
    let locked = tmp.path().join("locked");
    let work = locked.join("project");
    std::fs::create_dir_all(&work).expect("mkdir work");
    let run = seed_run_for_work_dir(&home_logs, RUN_NEWEST, &work);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).expect("lock");
    let unreadable = work.try_exists().is_err();

    let (removed, _) =
        super::log_gc_buckets::prune_all_log_buckets(&home_logs, &NO_LIMITS, None, None);

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).expect("unlock");
    if unreadable {
        assert_eq!(removed, 0);
        assert!(
            run.is_dir(),
            "an unreadable parent is not a missing project"
        );
    }
}

#[test]
fn prune_logs_after_run_created_removes_ephemeral_workspace_hash_buckets() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let other = malvin_home_logs_root().join(OTHER_HASH);
        let run = other.join(RUN_NEWEST);
        std::fs::create_dir_all(&run).expect("seed run");
        let gone = std::env::temp_dir().join("malvin_gc_missing_workspace_path");
        let _ = std::fs::remove_dir_all(&gone);
        write_work_dir_manifest(&run, &gone).expect("manifest to missing path");

        let protect = work.join("__no_active_run__");
        prune_logs_after_run_created(work, &protect);

        assert!(
            !other.exists(),
            "hash bucket whose work_dir no longer exists must be collected"
        );
    });
}

#[test]
fn prune_run_dirs_never_deletes_protected_active_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs = tmp.path().join("logs");
    let old = logs.join(RUN_OLDEST);
    let current = logs.join(RUN_NEWEST);
    std::fs::create_dir_all(&old).expect("old");
    std::fs::create_dir_all(&current).expect("current");
    let mut runs = vec![current.clone(), old.clone()];
    let config = LogsGcConfig {
        max_count: Some(1),
        max_age_days: None,
        max_bytes: None,
    };
    let removed = prune_run_dirs(&mut runs, &config, Some(current.as_path())).removed;
    assert_eq!(removed, 1);
    assert!(!old.exists());
    assert!(
        current.is_dir(),
        "protected in-progress run must survive count-cap prune"
    );
}

#[test]
fn prune_leaves_nonempty_foreign_bucket_with_non_run_content() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let config_path = crate::malvin_config_path(work);
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir config parent");
        }
        std::fs::write(&config_path, AGE_ONLY_TOML).expect("write config");
        let other = malvin_home_logs_root().join(OTHER_HASH);
        let old = other.join(RUN_OLD_AGE);
        std::fs::create_dir_all(&old).expect("seed run");
        write_work_dir_manifest(&old, work).expect("manifest");
        std::fs::create_dir_all(other.join("hand_notes")).expect("notes");

        let protect = work.join("__no_active_run__");
        prune_logs_after_run_created(work, &protect);

        assert!(!old.exists());
        assert!(
            other.join("hand_notes").is_dir(),
            "non-run content keeps the foreign bucket"
        );
    });
}
