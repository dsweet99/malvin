//! Snapshot and restore workspace `.malvin_checks` for long-running CLI workflows.

use std::path::{Path, PathBuf};

use super::dotfile_backup::{
    DotfileBackupLabels, allocate_backup_dir as allocate_dotfile_backup_dir,
    remove_if_exists as remove_dotfile_if_exists,
};
use super::run_id::random_alnum;

const LABELS: DotfileBackupLabels = DotfileBackupLabels {
    mkdir: "malvin_checks backup mkdir",
    collision: "malvin_checks backup mkdir",
    restore: "malvin_checks restore",
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MalvinChecksBackup {
    Missing,
    Present(PathBuf),
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_checks_if_present(
    work_dir: &Path,
) -> Result<MalvinChecksBackup, String> {
    backup_workspace_malvin_checks_if_present_with_id(work_dir, |_| random_alnum(5))
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_checks_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<MalvinChecksBackup, String> {
    let src = work_dir.join(crate::repo_gates::MALVIN_CHECKS_FILE);
    if !src.is_file() {
        return Ok(MalvinChecksBackup::Missing);
    }
    let root = crate::prompts::user_home_dir()
        .join(".malvin")
        .join("malvin_checks_snapshots");
    let dest_dir = allocate_dotfile_backup_dir(&root, &mut generate_id, &LABELS)?;
    let dest_file = dest_dir.join(crate::repo_gates::MALVIN_CHECKS_FILE);
    std::fs::copy(&src, &dest_file).map_err(|e| format!(".malvin_checks backup copy: {e}"))?;
    Ok(MalvinChecksBackup::Present(dest_file))
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_checks_backup(
    work_dir: &Path,
    backup: &MalvinChecksBackup,
) -> Result<(), String> {
    let dst = work_dir.join(crate::repo_gates::MALVIN_CHECKS_FILE);
    match backup {
        MalvinChecksBackup::Missing => remove_dotfile_if_exists(&dst, LABELS.restore),
        MalvinChecksBackup::Present(backup_path) => {
            std::fs::copy(backup_path, &dst).map_err(|e| format!("malvin_checks restore: {e}"))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    use std::path::Path;

    use super::{
        MalvinChecksBackup, backup_workspace_malvin_checks_if_present,
        backup_workspace_malvin_checks_if_present_with_id, restore_workspace_malvin_checks_backup,
    };
    use crate::artifacts::dotfile_backup::test_support::with_isolated_home;

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
            std::fs::write(work.join(".malvin_checks"), "ORIGINAL\n").unwrap();
            let backup = backup_workspace_malvin_checks_if_present(work).unwrap();
            let MalvinChecksBackup::Present(path) = &backup else {
                panic!("expected backup path");
            };
            assert!(path.is_file());
            std::fs::write(work.join(".malvin_checks"), "MODIFIED\n").unwrap();
            restore_workspace_malvin_checks_backup(work, &backup).unwrap();
            assert_eq!(
                std::fs::read_to_string(work.join(".malvin_checks")).unwrap(),
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
        std::fs::write(work.join(".malvin_checks"), "CREATED\n").unwrap();
        restore_workspace_malvin_checks_backup(&work, &backup).unwrap();
        assert!(!work.join(".malvin_checks").exists());
    }

    #[test]
    fn restore_workspace_malvin_checks_backup_removes_created_directory_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path().join("repo");
        std::fs::create_dir_all(&work).unwrap();
        let backup = backup_workspace_malvin_checks_if_present(&work).unwrap();
        let p = work.join(".malvin_checks");
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
                .join("malvin_checks_snapshots");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::create_dir_all(dir.join("aaaaa")).unwrap();

            std::fs::write(work.join(".malvin_checks"), "ORIGINAL\n").unwrap();
            let backup = backup_workspace_malvin_checks_if_present_with_id(work, |attempt| {
                if attempt == 0 {
                    "aaaaa".to_string()
                } else {
                    "bbbbb".to_string()
                }
            })
            .unwrap();

            let MalvinChecksBackup::Present(path) = &backup else {
                panic!("expected backup path");
            };

            assert_eq!(path.parent(), Some(dir.join("bbbbb").as_path()));
            assert!(dir.join("bbbbb").join(".malvin_checks").is_file());
            assert!(!dir.join("aaaaa").join(".malvin_checks").exists());
        });
    }

    #[test]
    fn kiss_stringify_malvin_checks_backup_units() {
        let _ = stringify!(crate::artifacts::malvin_checks_backup::MalvinChecksBackup);
        let _ = stringify!(
            crate::artifacts::malvin_checks_backup::backup_workspace_malvin_checks_if_present
        );
        let _ = stringify!(crate::artifacts::malvin_checks_backup::allocate_backup_dir);
        let _ = stringify!(crate::artifacts::malvin_checks_backup::remove_if_exists);
        let _ = stringify!(
            crate::artifacts::malvin_checks_backup::restore_workspace_malvin_checks_backup
        );
    }
}
