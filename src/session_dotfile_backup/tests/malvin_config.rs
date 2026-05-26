#![allow(unsafe_code)]

use std::path::Path;

use crate::artifacts::{
    MalvinConfigBackup, backup_workspace_malvin_config_if_present,
    backup_workspace_malvin_config_if_present_with_id, restore_workspace_malvin_config_backup,
};
use crate::test_utils::with_isolated_home;
use crate::{MALVIN_CONFIG_REL, seed_malvin_config};

#[test]
fn malvin_config_backup_skips_when_workspace_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("empty");
    std::fs::create_dir_all(&work).unwrap();
    assert_eq!(
        backup_workspace_malvin_config_if_present(&work).unwrap(),
        MalvinConfigBackup::Missing
    );
}

#[test]
fn malvin_config_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_config_if_present(work).unwrap();
        let MalvinConfigBackup::Present(path) = &backup else {
            panic!("expected backup path");
        };
        assert!(path.is_file());
        seed_malvin_config(work, "MODIFIED\n");
        restore_workspace_malvin_config_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(MALVIN_CONFIG_REL)).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn malvin_config_backup_missing_restores_by_removing_created_workspace_file() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_malvin_config_if_present(&work).unwrap();
    seed_malvin_config(&work, "CREATED\n");
    restore_workspace_malvin_config_backup(&work, &backup).unwrap();
    assert!(!work.join(MALVIN_CONFIG_REL).exists());
}

#[test]
fn restore_workspace_malvin_config_backup_removes_created_directory_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_malvin_config_if_present(&work).unwrap();
    std::fs::create_dir_all(work.join(".malvin")).unwrap();
    let p = work.join(MALVIN_CONFIG_REL);
    std::fs::create_dir(&p).unwrap();
    restore_workspace_malvin_config_backup(&work, &backup).unwrap();
    assert!(!p.exists());
}

#[test]
fn malvin_config_backup_retries_on_existing_collision() {
    with_isolated_home(|work| {
        let home = std::env::var_os("HOME").unwrap();
        let dir = Path::new(&home)
            .join(".malvin")
            .join("malvin_config_snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("aaaaa")).unwrap();

        seed_malvin_config(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_config_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();

        let MalvinConfigBackup::Present(path) = &backup else {
            panic!("expected backup path");
        };

        assert_eq!(
            path.as_path(),
            dir.join("bbbbb").join(MALVIN_CONFIG_REL).as_path()
        );
        assert!(path.is_file());
        assert!(!dir.join("aaaaa").join(MALVIN_CONFIG_REL).exists());
    });
}
