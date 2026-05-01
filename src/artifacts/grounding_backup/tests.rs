#![allow(unsafe_code)]

use std::path::Path;

use super::{
    GroundingBackup,
    backup_workspace_grounding_if_present,
    restore_workspace_grounding,
    restore_workspace_kissconfig,
    backup_workspace_grounding_if_present_with_id,
};

fn with_isolated_home<F>(f: F)
where
    F: FnOnce(&Path),
{
    let _lock = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let old_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &home);
    }
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    f(&work);
    unsafe {
        match old_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
    }
}

#[test]
fn grounding_backup_skips_when_workspace_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("empty");
    std::fs::create_dir_all(&work).unwrap();
    assert_eq!(
        backup_workspace_grounding_if_present(&work).unwrap(),
        GroundingBackup::Missing
    );
}

#[test]
fn grounding_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        std::fs::write(work.join("grounding.md"), "ORIGINAL\n").unwrap();
        let backup = backup_workspace_grounding_if_present(work).unwrap();
        let GroundingBackup::Present(backup_files) = &backup else {
            panic!("expected backup path");
        };
        assert!(backup_files.kissconfig.is_none());
        assert!(backup_files.grounding.is_some());
        std::fs::write(work.join("grounding.md"), "MUTATED\n").unwrap();
        restore_workspace_grounding(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("grounding.md")).unwrap(),
            "ORIGINAL\n"
        );
        assert!(!work.join(".kissconfig").exists());
    });
}

#[test]
fn kissconfig_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        std::fs::write(work.join(".kissconfig"), "KISS=ORIGINAL\n").unwrap();
        let backup = backup_workspace_grounding_if_present(work).unwrap();
        let GroundingBackup::Present(backup_files) = &backup else {
            panic!("expected backup path");
        };
        assert!(backup_files.grounding.is_none());
        assert!(backup_files.kissconfig.is_some());
        std::fs::write(work.join(".kissconfig"), "KISS=MODIFIED\n").unwrap();
        restore_workspace_grounding(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".kissconfig")).unwrap(),
            "KISS=ORIGINAL\n"
        );
        assert!(!work.join("grounding.md").exists());
    });
}

#[test]
fn grounding_backup_missing_restores_by_removing_created_workspace_files() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_grounding_if_present(&work).unwrap();
    std::fs::write(work.join("grounding.md"), "CREATED\n").unwrap();
    std::fs::write(work.join(".kissconfig"), "CREATED\n").unwrap();
    restore_workspace_grounding(&work, &backup).unwrap();
    assert!(!work.join("grounding.md").exists());
    assert!(!work.join(".kissconfig").exists());
}

#[test]
fn restore_workspace_kissconfig_missing_backup_removes_created_kissconfig() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_grounding_if_present(&work).unwrap();
    std::fs::write(work.join(".kissconfig"), "CREATED\n").unwrap();
    restore_workspace_kissconfig(&work, &backup).unwrap();
    assert!(!work.join(".kissconfig").exists());
}

#[test]
fn restore_workspace_grounding_removes_created_directory_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_grounding_if_present(&work).unwrap();
    let grounding = work.join("grounding.md");
    let kissconfig = work.join(".kissconfig");
    std::fs::create_dir(&grounding).unwrap();
    std::fs::create_dir(&kissconfig).unwrap();
    restore_workspace_grounding(&work, &backup).unwrap();
    assert!(!grounding.exists());
    assert!(!kissconfig.exists());
}

#[test]
fn kissconfig_is_restored_even_when_grounding_is_missing() {
    with_isolated_home(|work| {
        std::fs::write(work.join(".kissconfig"), "KISS=ORIGINAL\n").unwrap();
        let backup = backup_workspace_grounding_if_present(work).unwrap();
        let GroundingBackup::Present(backup_files) = &backup else {
            panic!("expected backup path");
        };
        assert!(backup_files.kissconfig.is_some());
        std::fs::remove_file(work.join(".kissconfig")).unwrap();
        restore_workspace_grounding(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".kissconfig")).unwrap(),
            "KISS=ORIGINAL\n"
        );
    });
}

#[test]
fn kiss_stringify_grounding_backup_units() {
    let _ = stringify!(crate::artifacts::grounding_backup::ProtectedWorkspaceFiles);
    let _ = stringify!(crate::artifacts::grounding_backup::backup_workspace_file);
    let _ = stringify!(crate::artifacts::grounding_backup::restore_workspace_file);
}

#[test]
fn grounding_backup_retries_on_existing_collision() {
    with_isolated_home(|work| {
        let home = std::env::var_os("HOME").unwrap();
        let groundings = Path::new(&home).join(".malvin").join("groundings");
        std::fs::create_dir_all(&groundings).unwrap();
        std::fs::create_dir_all(groundings.join("aaaaa")).unwrap();

        std::fs::write(work.join("grounding.md"), "ORIGINAL\n").unwrap();
        let backup = backup_workspace_grounding_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();

        let GroundingBackup::Present(backup_files) = &backup else {
            panic!("expected backup path");
        };

        assert_eq!(
            backup_files.grounding.as_ref().unwrap().parent(),
            Some(groundings.join("bbbbb").as_path())
        );
        assert!(groundings.join("bbbbb").join("grounding.md").is_file());
        assert!(!groundings.join("aaaaa").join("grounding.md").exists());
    });
}
