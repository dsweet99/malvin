//! Snapshot and restore workspace `grounding.md` for long-running CLI workflows.

use std::path::{Path, PathBuf};

use super::run_id::random_alnum;

#[path = "grounding_backup_impl.rs"]
mod grounding_backup_impl;
use grounding_backup_impl::backup_workspace_grounding_if_present_with_id;

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
    backup_workspace_grounding_if_present_with_id(work_dir, |_| random_alnum(5))
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
            restore_workspace_file(work_dir, ".kissconfig", backup_files.kissconfig.as_deref())?;
            Ok(())
        }
    }
}

pub fn restore_workspace_kissconfig(
    work_dir: &Path,
    backup: &GroundingBackup,
) -> Result<(), String> {
    match backup {
        GroundingBackup::Missing => Ok(()),
        GroundingBackup::Present(backup_files) => {
            restore_workspace_file(work_dir, ".kissconfig", backup_files.kissconfig.as_deref())
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
mod tests;
