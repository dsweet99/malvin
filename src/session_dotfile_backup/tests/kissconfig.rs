#![allow(unsafe_code)]

use std::path::Path;

use crate::artifacts::{
    KissConfigBackup, backup_workspace_kissconfig_if_present,
    backup_workspace_kissconfig_if_present_with_id, restore_workspace_kissconfig_backup,
};
use crate::test_utils::with_isolated_home;

#[cfg(unix)]
fn install_fake_kiss_clamp_script(bin_dir: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let kiss = bin_dir.join("kiss");
    std::fs::write(
        &kiss,
        "#!/bin/sh\ncd \"$PWD\"\nprintf '%s\\n' '[gate]' 'test_coverage_threshold = 0' > .kissconfig\nexit 0\n",
    )
    .unwrap();
    let mut perms = std::fs::metadata(&kiss).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&kiss, perms).unwrap();
}

#[cfg(unix)]
#[test]
fn snapshot_runs_kiss_clamp_before_backing_up_kissconfig() {
    use crate::artifacts::SessionDotfileBackups;
    use crate::repo_checks::set_fake_command_dir;

    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    std::fs::write(work.join("main.rs"), "fn main() {}").unwrap();

    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_kiss_clamp_script(bin_dir.path());
    let _guard = set_fake_command_dir(bin_dir.path());

    let backups = SessionDotfileBackups::snapshot(work).unwrap();
    assert!(matches!(
        backups.kissconfig,
        crate::artifacts::KissConfigBackup::Present(_)
    ));
    assert!(work.join(".kissconfig").is_file());
}

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
        let KissConfigBackup::Present(payload) = &backup else {
            panic!("expected backup path");
        };
        assert!(payload.backup_path.is_file());
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
        let dir = Path::new(&home)
            .join(".malvin")
            .join("snapshots")
            .join("kissconfig");
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

        let KissConfigBackup::Present(payload) = &backup else {
            panic!("expected backup path");
        };

        assert_eq!(payload.backup_path.parent(), Some(dir.join("bbbbb").as_path()));
        assert!(dir.join("bbbbb").join(".kissconfig").is_file());
        assert!(!dir.join("aaaaa").join(".kissconfig").exists());
    });
}
