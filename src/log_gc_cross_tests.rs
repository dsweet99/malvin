use super::*;
use super::log_gc_prune::prune_run_dirs;
use crate::log_gc_config::LogsGcConfig;
use crate::workspace_paths::{malvin_home_logs_root, malvin_logs_root, write_work_dir_manifest};

const RUN_OLD_AGE: &str = "20200101_000000_oldrun01";
const RUN_OLDEST: &str = "20260101_000000_aaaaaaa1";
const RUN_NEWEST: &str = "20260103_000000_ccccccc3";
const OTHER_HASH: &str = "deadbeefdeadbeef";
const ORPHAN_HASH: &str = "aaaaaaaaaaaaaaaa";
const AGE_ONLY_TOML: &str = "[logs]\nmax_count = 0\nmax_age_days = 30\nmax_bytes = \"\"\n";

#[test]
fn prune_logs_after_run_created_applies_retention_across_hash_buckets() {
    crate::test_utils::with_isolated_home(|work| {
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
    crate::test_utils::with_isolated_home(|work| {
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

#[test]
fn prune_logs_after_run_created_removes_ephemeral_workspace_hash_buckets() {
    crate::test_utils::with_isolated_home(|work| {
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
        max_count: 1,
        max_age_days: 0,
        max_bytes: None,
    };
    let (removed, _) = prune_run_dirs(&mut runs, &config, Some(current.as_path()));
    assert_eq!(removed, 1);
    assert!(!old.exists());
    assert!(
        current.is_dir(),
        "protected in-progress run must survive count-cap prune"
    );
}

#[test]
fn prune_leaves_nonempty_foreign_bucket_with_non_run_content() {
    crate::test_utils::with_isolated_home(|work| {
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
