use crate::workspace_paths::{
    canonical_work_dir_for_logs, find_malvin_logs_root, is_malvin_workspace, malvin_advice_path,
    malvin_checks_path, malvin_config_path, malvin_home_config_path, malvin_home_logs_root,
    malvin_home_snapshots_root, malvin_logs_root, malvin_user_home_root, read_work_dir_manifest,
    remove_legacy_malvin_checks_file, snapshot_category_dir, write_work_dir_manifest,
    workspace_logs_hash, MALVIN_CHECKS_REL, MALVIN_DIR,
    MALVIN_USER_HOME_DIR,
};

#[test]
fn path_helpers_and_workspace_marker() {
    let _ = crate::seed_malvin_config;
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    assert_eq!(malvin_checks_path(w), w.join(MALVIN_CHECKS_REL));
    assert_eq!(malvin_advice_path(w), w.join(".malvin/advice.md"));
    assert_eq!(malvin_config_path(w), malvin_home_config_path());
    assert!(!is_malvin_workspace(w));
    std::fs::create_dir_all(w.join(MALVIN_DIR)).unwrap();
    assert!(is_malvin_workspace(w));
}

#[test]
fn workspace_logs_hash_is_stable_hex() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path().join("proj");
    std::fs::create_dir_all(&w).unwrap();
    let h1 = workspace_logs_hash(&w);
    let h2 = workspace_logs_hash(&w);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 16);
    assert!(h1.bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn workspace_logs_hash_differs_for_different_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a");
    let b = tmp.path().join("b");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    assert_ne!(workspace_logs_hash(&a), workspace_logs_hash(&b));
}

#[test]
fn malvin_user_home_root_uses_malvin_home_dir() {
    let root = malvin_user_home_root();
    assert!(root.ends_with(MALVIN_USER_HOME_DIR));
    assert!(root.starts_with(crate::user_home_dir()));
}

#[test]
fn malvin_logs_root_lives_under_home_not_workspace() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path().join("ws");
    std::fs::create_dir_all(&w).unwrap();
    let root = malvin_logs_root(&w);
    assert!(root.starts_with(malvin_home_logs_root()));
    assert!(!root.starts_with(w));
}

#[test]
fn malvin_snapshots_root_lives_under_home() {
    let root = malvin_home_snapshots_root();
    assert!(root.ends_with(".malvin/snapshots") || root.ends_with(".malvin\\snapshots"));
    assert_eq!(snapshot_category_dir("gitignore"), root.join("gitignore"));
}

#[test]
fn find_malvin_logs_root_none_until_bucket_exists() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path().join("fresh");
    std::fs::create_dir_all(&w).unwrap();
    assert_eq!(find_malvin_logs_root(&w), None);
    let bucket = malvin_logs_root(&w);
    std::fs::create_dir_all(&bucket).unwrap();
    assert_eq!(find_malvin_logs_root(&w).as_deref(), Some(bucket.as_path()));
}

#[test]
fn work_dir_manifest_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let ws = tmp.path().join("ws");
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::create_dir_all(&run).unwrap();
    write_work_dir_manifest(&run, &ws).unwrap();
    let read = read_work_dir_manifest(&run).expect("manifest");
    assert_eq!(read, canonical_work_dir_for_logs(&ws));
}

#[test]
fn remove_legacy_malvin_checks_file_deletes_legacy_not_layout_checks() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    std::fs::write(w.join(".malvin_checks"), "legacy\n").unwrap();
    std::fs::create_dir_all(w.join(MALVIN_DIR)).unwrap();
    std::fs::write(malvin_checks_path(w), "current\n").unwrap();
    remove_legacy_malvin_checks_file(w);
    assert!(!w.join(".malvin_checks").exists());
    assert_eq!(std::fs::read_to_string(malvin_checks_path(w)).unwrap(), "current\n");
}

#[test]
fn home_malvin_config_delete_blocked_without_test_mutation_flag() {
    use crate::artifacts::SessionDotfileBackups;
    use crate::malvin_config_file::{open_malvin_config, write_config_value};

    crate::test_utils::with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        assert!(!cfg.exists());
        let backup = SessionDotfileBackups::snapshot(work).expect("snapshot");
        open_malvin_config(work).expect("ensure default");
        assert!(cfg.is_file());
        crate::test_utils::revoke_home_malvin_config_mutation_for_test();
        backup.restore_excluding_malvin_checks(work).expect("restore");
        assert!(
            cfg.is_file(),
            "without mutation flag, Missing restore must not delete home config"
        );
        let value: toml::Value = toml::from_str("mem_limit_gb = 99").expect("toml");
        assert!(
            write_config_value(&cfg, &value).is_err(),
            "write must fail without mutation consent"
        );
        crate::test_utils::allow_home_malvin_config_mutation_for_test();
    });
}
