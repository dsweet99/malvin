//! Snapshot and restore workspace `grounding.md` for long-running CLI workflows.

use std::path::{Path, PathBuf};

use super::random_alnum;

/// When `work_dir/grounding.md` exists, copy it to `~/.malvin/groundings/<id>/grounding.md` and return that path.
///
/// Returns `None` when there is no workspace `grounding.md` to snapshot.
///
/// # Errors
///
/// Returns an error string if the destination directory cannot be created or the file cannot be copied.
pub fn backup_workspace_grounding_if_present(work_dir: &Path) -> Result<Option<PathBuf>, String> {
    let src = work_dir.join("grounding.md");
    if !src.is_file() {
        return Ok(None);
    }
    let id = random_alnum(5);
    let dest_dir = crate::prompts::user_home_dir()
        .join(".malvin")
        .join("groundings")
        .join(&id);
    std::fs::create_dir_all(&dest_dir).map_err(|e| format!("grounding backup mkdir: {e}"))?;
    let dest = dest_dir.join("grounding.md");
    std::fs::copy(&src, &dest).map_err(|e| format!("grounding backup copy: {e}"))?;
    Ok(Some(dest))
}

/// Overwrite `work_dir/grounding.md` from a file returned by [`backup_workspace_grounding_if_present`].
///
/// # Errors
///
/// Returns an error string if the backup file cannot be read or `grounding.md` cannot be written.
pub fn restore_workspace_grounding(work_dir: &Path, backup_file: &Path) -> Result<(), String> {
    let dst = work_dir.join("grounding.md");
    std::fs::copy(backup_file, &dst).map_err(|e| format!("grounding restore: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{backup_workspace_grounding_if_present, restore_workspace_grounding};

    #[test]
    fn grounding_backup_skips_when_workspace_file_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path().join("empty");
        std::fs::create_dir_all(&work).unwrap();
        assert!(backup_workspace_grounding_if_present(&work)
            .unwrap()
            .is_none());
    }

    #[test]
    #[allow(unsafe_code)]
    fn grounding_backup_round_trip_restores_workspace_file() {
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
        std::fs::write(work.join("grounding.md"), "ORIGINAL\n").unwrap();
        let backup = backup_workspace_grounding_if_present(&work)
            .unwrap()
            .expect("backup path");
        assert!(backup.starts_with(&home));
        assert!(backup.is_file());
        std::fs::write(work.join("grounding.md"), "MUTATED\n").unwrap();
        restore_workspace_grounding(&work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("grounding.md")).unwrap(),
            "ORIGINAL\n"
        );
        unsafe {
            match old_home {
                Some(h) => std::env::set_var("HOME", h),
                None => std::env::remove_var("HOME"),
            }
        }
    }
}
