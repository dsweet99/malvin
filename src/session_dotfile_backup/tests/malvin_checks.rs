#![allow(unsafe_code)]

use std::path::Path;

use crate::artifacts::{
    MalvinChecksBackup, backup_workspace_malvin_checks_if_present,
    backup_workspace_malvin_checks_if_present_with_id, restore_workspace_malvin_checks_backup,
};
use crate::test_utils::with_isolated_home;
use crate::seed_malvin_checks;

#[test]
fn malvin_checks_backup_skips_when_workspace_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("empty");
    std::fs::create_dir_all(&work).unwrap();
    assert_eq!(
        backup_workspace_malvin_checks_if_present(&work).unwrap(),
        MalvinChecksBackup::Missing
    );
}

#[test]
fn malvin_checks_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        seed_malvin_checks(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_checks_if_present(work).unwrap();
        let MalvinChecksBackup::Present(payload) = &backup else {
            panic!("expected backup path");
        };
        assert!(payload.backup_path.is_file());
        seed_malvin_checks(work, "MODIFIED\n");
        restore_workspace_malvin_checks_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".malvin/checks")).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn malvin_checks_backup_missing_restores_by_removing_created_workspace_file() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_malvin_checks_if_present(&work).unwrap();
    seed_malvin_checks(&work, "CREATED\n");
    restore_workspace_malvin_checks_backup(&work, &backup).unwrap();
    assert!(!work.join(".malvin/checks").exists());
}

#[test]
fn restore_workspace_malvin_checks_backup_removes_created_directory_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_malvin_checks_if_present(&work).unwrap();
    std::fs::create_dir_all(work.join(".malvin")).unwrap();
    let p = work.join(".malvin/checks");
    std::fs::create_dir(&p).unwrap();
    restore_workspace_malvin_checks_backup(&work, &backup).unwrap();
    assert!(!p.exists());
}

#[test]
fn malvin_checks_backup_retries_on_existing_collision() {
    with_isolated_home(|work| {
        let home = std::env::var_os("HOME").unwrap();
        let dir = Path::new(&home)
            .join(".malvin")
            .join("snapshots")
            .join("malvin_checks");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("aaaaa")).unwrap();

        seed_malvin_checks(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_checks_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();

        let MalvinChecksBackup::Present(payload) = &backup else {
            panic!("expected backup path");
        };

        assert_eq!(
            payload.backup_path.as_path(),
            dir.join("bbbbb").join(".malvin/checks").as_path()
        );
        assert!(payload.backup_path.is_file());
        assert!(!dir.join("aaaaa").join(".malvin/checks").exists());
    });
}
