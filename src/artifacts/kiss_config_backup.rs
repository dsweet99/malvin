//! Snapshot and restore workspace `.kissconfig` for long-running CLI workflows.

use std::path::{Path, PathBuf};

use super::dotfile_backup::{
    DotfileBackupLabels, allocate_backup_dir as allocate_dotfile_backup_dir,
    remove_if_exists as remove_dotfile_if_exists,
};
use super::run_id::random_alnum;

const LABELS: DotfileBackupLabels = DotfileBackupLabels {
    mkdir: "kissconfig backup mkdir",
    collision: "kissconfig backup mkdir",
    restore: "kissconfig restore",
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KissConfigBackup {
    Missing,
    Present(PathBuf),
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissconfig_if_present(work_dir: &Path) -> Result<KissConfigBackup, String> {
    backup_workspace_kissconfig_if_present_with_id(work_dir, |_| random_alnum(5))
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissconfig_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<KissConfigBackup, String> {
    let kissconfig_src = work_dir.join(".kissconfig");
    if !kissconfig_src.is_file() {
        return Ok(KissConfigBackup::Missing);
    }
    let root = crate::prompts::user_home_dir()
        .join(".malvin")
        .join("kissconfigs");
    let dest_dir = allocate_dotfile_backup_dir(&root, &mut generate_id, &LABELS)?;
    let dest_file = dest_dir.join(".kissconfig");
    std::fs::copy(&kissconfig_src, &dest_file)
        .map_err(|e| format!(".kissconfig backup copy: {e}"))?;
    Ok(KissConfigBackup::Present(dest_file))
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_kissconfig_backup(
    work_dir: &Path,
    backup: &KissConfigBackup,
) -> Result<(), String> {
    match backup {
        KissConfigBackup::Missing => {
            remove_dotfile_if_exists(&work_dir.join(".kissconfig"), LABELS.restore)
        }
        KissConfigBackup::Present(backup_path) => {
            let dst = work_dir.join(".kissconfig");
            std::fs::copy(backup_path, &dst).map_err(|e| format!("kissconfig restore: {e}"))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    use std::path::Path;

    use super::{
        KissConfigBackup, backup_workspace_kissconfig_if_present,
        backup_workspace_kissconfig_if_present_with_id, restore_workspace_kissconfig_backup,
    };
    use crate::artifacts::dotfile_backup::test_support::with_isolated_home;

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

    #[test]
    fn kiss_stringify_kiss_config_backup_units() {
        let _ = stringify!(crate::artifacts::kiss_config_backup::KissConfigBackup);
        let _ = stringify!(
            crate::artifacts::kiss_config_backup::backup_workspace_kissconfig_if_present
        );
        let _ = stringify!(crate::artifacts::kiss_config_backup::allocate_backup_dir);
        let _ = stringify!(crate::artifacts::kiss_config_backup::remove_if_exists);
        let _ =
            stringify!(crate::artifacts::kiss_config_backup::restore_workspace_kissconfig_backup);
    }
}
