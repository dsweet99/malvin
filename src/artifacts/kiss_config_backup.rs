//! Snapshot and restore workspace `.kissconfig` for long-running CLI workflows.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::run_id::random_alnum;

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
    let dest_dir = allocate_backup_dir(&root, &mut generate_id)?;
    let dest_file = dest_dir.join(".kissconfig");
    std::fs::copy(&kissconfig_src, &dest_file)
        .map_err(|e| format!(".kissconfig backup copy: {e}"))?;
    Ok(KissConfigBackup::Present(dest_file))
}

fn allocate_backup_dir(
    root: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(root).map_err(|e| format!("kissconfig backup mkdir: {e}"))?;
    let mut tries = 0usize;
    while tries < 16 {
        let candidate = root.join(generate_id(tries));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                tries += 1;
            }
            Err(err) => return Err(format!("kissconfig backup mkdir: {err}")),
        }
    }
    Err("kissconfig backup mkdir: too many id collisions".to_string())
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_kissconfig_backup(
    work_dir: &Path,
    backup: &KissConfigBackup,
) -> Result<(), String> {
    match backup {
        KissConfigBackup::Missing => remove_if_exists(&work_dir.join(".kissconfig")),
        KissConfigBackup::Present(backup_path) => {
            let dst = work_dir.join(".kissconfig");
            std::fs::copy(backup_path, &dst).map_err(|e| format!("kissconfig restore: {e}"))?;
            Ok(())
        }
    }
}

fn remove_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        let metadata = std::fs::metadata(path).map_err(|e| format!("kissconfig restore: {e}"))?;
        if metadata.is_dir() {
            std::fs::remove_dir_all(path).map_err(|e| format!("kissconfig restore: {e}"))?;
        } else {
            std::fs::remove_file(path).map_err(|e| format!("kissconfig restore: {e}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    use std::path::Path;

    use super::{
        KissConfigBackup, backup_workspace_kissconfig_if_present,
        backup_workspace_kissconfig_if_present_with_id, restore_workspace_kissconfig_backup,
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
        let _ = stringify!(crate::artifacts::kiss_config_backup::backup_workspace_kissconfig_if_present);
        let _ = stringify!(crate::artifacts::kiss_config_backup::restore_workspace_kissconfig_backup);
    }
}
