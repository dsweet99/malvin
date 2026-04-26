//! Snapshot and restore workspace `grounding.md` for long-running CLI workflows.

use std::path::{Path, PathBuf};

use super::run_id::random_alnum;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundingBackup {
    Missing,
    Present(ProtectedWorkspaceFiles),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedWorkspaceFiles {
    grounding: Option<PathBuf>,
    kissconfig: Option<PathBuf>,
}

/// When protected files exist, copy each one to `~/.malvin/groundings/<id>/` and
/// return their restored paths.
///
/// # Errors
///
/// Returns an error string if the destination directory cannot be created or the file cannot be copied.
pub fn backup_workspace_grounding_if_present(work_dir: &Path) -> Result<GroundingBackup, String> {
    let grounding_src = work_dir.join("grounding.md");
    let kissconfig_src = work_dir.join(".kissconfig");
    if !grounding_src.is_file() && !kissconfig_src.is_file() {
        return Ok(GroundingBackup::Missing);
    }
    let id = random_alnum(5);
    let dest_dir = crate::prompts::user_home_dir()
        .join(".malvin")
        .join("groundings")
        .join(&id);
    std::fs::create_dir_all(&dest_dir).map_err(|e| format!("grounding backup mkdir: {e}"))?;

    let grounding = backup_workspace_file(&grounding_src, &dest_dir, "grounding.md")?;
    let kissconfig = backup_workspace_file(&kissconfig_src, &dest_dir, ".kissconfig")?;

    Ok(GroundingBackup::Present(ProtectedWorkspaceFiles {
        grounding,
        kissconfig,
    }))
}

fn backup_workspace_file(
    source: &Path,
    destination_dir: &Path,
    filename: &str,
) -> Result<Option<PathBuf>, String> {
    if !source.is_file() {
        return Ok(None);
    }
    let destination = destination_dir.join(filename);
    std::fs::copy(source, &destination)
        .map_err(|e| format!("{filename} backup copy: {e}"))?;
    Ok(Some(destination))
}

/// Overwrite protected workspace files from data returned by
/// [`backup_workspace_grounding_if_present`].
///
/// # Errors
///
/// Returns an error string if a backup file cannot be read or a destination file
/// cannot be written.
pub fn restore_workspace_grounding(
    work_dir: &Path,
    backup: &GroundingBackup,
) -> Result<(), String> {
    match backup {
        GroundingBackup::Missing => Ok(()),
        GroundingBackup::Present(backup_files) => {
            restore_workspace_file(work_dir, "grounding.md", backup_files.grounding.as_deref())?;
            restore_workspace_file(
                work_dir,
                ".kissconfig",
                backup_files.kissconfig.as_deref(),
            )?;
            Ok(())
        }
    }
}

fn restore_workspace_file(
    work_dir: &Path,
    filename: &str,
    backup: Option<&Path>,
) -> Result<(), String> {
    let dst = work_dir.join(filename);
    if let Some(backup_path) = backup {
        std::fs::copy(backup_path, &dst).map_err(|e| format!("grounding restore: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        GroundingBackup, backup_workspace_grounding_if_present, restore_workspace_grounding,
    };
    use std::path::Path;

    #[allow(unsafe_code)]
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
    #[allow(unsafe_code)]
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
    #[allow(unsafe_code)]
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
    fn grounding_backup_missing_restores_by_preserving_workspace_files() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path().join("repo");
        std::fs::create_dir_all(&work).unwrap();
        let backup = backup_workspace_grounding_if_present(&work).unwrap();
        std::fs::write(work.join("grounding.md"), "CREATED\n").unwrap();
        std::fs::write(work.join(".kissconfig"), "CREATED\n").unwrap();
        restore_workspace_grounding(&work, &backup).unwrap();
        assert_eq!(std::fs::read_to_string(work.join("grounding.md")).unwrap(), "CREATED\n");
        assert_eq!(std::fs::read_to_string(work.join(".kissconfig")).unwrap(), "CREATED\n");
    }

    #[test]
    #[allow(unsafe_code)]
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
}
