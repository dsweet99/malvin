#![allow(unsafe_code)]

use std::path::Path;

use crate::test_utils::with_isolated_home;
use crate::artifacts::{
    KissConfigBackup, backup_workspace_kissconfig_if_present,
    backup_workspace_kissconfig_if_present_with_id, restore_workspace_kissconfig_backup,
};

#[test]
fn kissconfig_backup_skips_when_workspace_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("empty");
    std::fs::create_dir_all(&work).unwrap();
    assert_eq!(
        backup_workspace_kissconfig_if_present(&work).unwrap(),
        KissConfigBackup::Missing
    );
}

#[test]
fn kissconfig_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        std::fs::write(work.join(".kissconfig"), "KISS=ORIGINAL\n").unwrap();
        let backup = backup_workspace_kissconfig_if_present(work).unwrap();
        let KissConfigBackup::Present(path) = &backup else {
            panic!("expected backup path");
        };
        assert!(path.is_file());
        std::fs::write(work.join(".kissconfig"), "KISS=MODIFIED\n").unwrap();
        restore_workspace_kissconfig_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".kissconfig")).unwrap(),
            "KISS=ORIGINAL\n"
        );
    });
}

#[test]
fn kissconfig_backup_missing_restores_by_removing_created_workspace_file() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_kissconfig_if_present(&work).unwrap();
    std::fs::write(work.join(".kissconfig"), "CREATED\n").unwrap();
    restore_workspace_kissconfig_backup(&work, &backup).unwrap();
    assert!(!work.join(".kissconfig").exists());
}

#[test]
fn restore_workspace_kissconfig_backup_removes_created_directory_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_kissconfig_if_present(&work).unwrap();
    let kissconfig = work.join(".kissconfig");
    std::fs::create_dir(&kissconfig).unwrap();
    restore_workspace_kissconfig_backup(&work, &backup).unwrap();
    assert!(!kissconfig.exists());
}

#[test]
fn kissconfig_backup_retries_on_existing_collision() {
    with_isolated_home(|work| {
        let home = std::env::var_os("HOME").unwrap();
        let dir = Path::new(&home).join(".malvin").join("kissconfigs");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("aaaaa")).unwrap();

        std::fs::write(work.join(".kissconfig"), "KISS=ORIGINAL\n").unwrap();
        let backup = backup_workspace_kissconfig_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();

        let KissConfigBackup::Present(path) = &backup else {
            panic!("expected backup path");
        };

        assert_eq!(path.parent(), Some(dir.join("bbbbb").as_path()));
        assert!(dir.join("bbbbb").join(".kissconfig").is_file());
        assert!(!dir.join("aaaaa").join(".kissconfig").exists());
    });
}
