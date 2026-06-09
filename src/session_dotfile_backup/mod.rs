mod alloc;
mod slots;
mod wrappers;

use std::path::Path;

pub use wrappers::{
    backup_workspace_gitignore_if_present, backup_workspace_gitignore_if_present_with_id,
    backup_workspace_kissconfig_if_present, backup_workspace_kissconfig_if_present_with_id,
    backup_workspace_kissignore_if_present, backup_workspace_kissignore_if_present_with_id,
    backup_workspace_malvin_checks_if_present, backup_workspace_malvin_checks_if_present_with_id,
    backup_workspace_malvin_config_if_present, backup_workspace_malvin_config_if_present_with_id,
    backup_workspace_malvin_config_workspace_if_present,
    backup_workspace_malvin_config_workspace_if_present_with_id,
    restore_workspace_gitignore_backup, restore_workspace_kissconfig_backup,
    restore_workspace_kissignore_backup, restore_workspace_malvin_checks_backup,
    restore_workspace_malvin_config_backup, restore_workspace_malvin_config_workspace_backup,
};

use slots::{backup_slot, restore_slot};

/// Captured dotfile bytes at snapshot time plus the historical disk location under `~/.malvin`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotfileBackupPayload {
    pub backup_path: std::path::PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DotfileBackupState {
    Missing,
    Present(DotfileBackupPayload),
}

pub type KissConfigBackup = DotfileBackupState;
pub type MalvinChecksBackup = DotfileBackupState;
pub type KissignoreBackup = DotfileBackupState;
pub type MalvinConfigBackup = DotfileBackupState;
pub type MalvinConfigWorkspaceBackup = DotfileBackupState;
pub type GitignoreBackup = DotfileBackupState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDotfileParts {
    pub kissconfig: KissConfigBackup,
    pub malvin_checks: MalvinChecksBackup,
    pub kissignore: KissignoreBackup,
    pub malvin_config: MalvinConfigBackup,
    pub gitignore: GitignoreBackup,
    pub malvin_config_workspace: MalvinConfigWorkspaceBackup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDotfileBackups {
    pub kissconfig: KissConfigBackup,
    pub malvin_checks: MalvinChecksBackup,
    pub kissignore: KissignoreBackup,
    pub malvin_config: MalvinConfigBackup,
    pub gitignore: GitignoreBackup,
    pub malvin_config_workspace: MalvinConfigWorkspaceBackup,
}

impl SessionDotfileBackups {
    #[must_use]
    pub fn from_parts(parts: SessionDotfileParts) -> Self {
        Self {
            kissconfig: parts.kissconfig,
            malvin_checks: parts.malvin_checks,
            kissignore: parts.kissignore,
            malvin_config: parts.malvin_config,
            gitignore: parts.gitignore,
            malvin_config_workspace: parts.malvin_config_workspace,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn snapshot(work_dir: &Path) -> Result<Self, String> {
        Self::snapshot_with_id(work_dir, alloc::random_backup_id)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn snapshot_with_id(
        work_dir: &Path,
        mut generate_id: impl FnMut(usize) -> String,
    ) -> Result<Self, String> {
        Ok(Self {
            kissconfig: backup_slot(0, work_dir, &mut generate_id)?,
            malvin_checks: backup_slot(1, work_dir, &mut generate_id)?,
            kissignore: backup_slot(2, work_dir, &mut generate_id)?,
            malvin_config: backup_slot(3, work_dir, &mut generate_id)?,
            gitignore: backup_slot(4, work_dir, &mut generate_id)?,
            malvin_config_workspace: backup_slot(5, work_dir, &mut generate_id)?,
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn restore(&self, work_dir: &Path) -> Result<(), String> {
        restore_workspace_session_dotfiles(work_dir, self)
    }

    /// Restore kiss and malvin config dotfiles only; leave `.malvin/checks` unchanged.
    #[allow(clippy::missing_errors_doc)]
    pub fn restore_excluding_malvin_checks(&self, work_dir: &Path) -> Result<(), String> {
        restore_workspace_session_dotfiles_excluding_malvin_checks(work_dir, self)
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_session_dotfiles(
    work_dir: &Path,
    bundle: &SessionDotfileBackups,
) -> Result<(), String> {
    restore_workspace_session_dotfiles_excluding_malvin_checks(work_dir, bundle)?;
    restore_slot(work_dir, &bundle.malvin_checks, 1)
        .map(|()| crate::remove_legacy_malvin_checks_file(work_dir))
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_session_dotfiles_excluding_malvin_checks(
    work_dir: &Path,
    bundle: &SessionDotfileBackups,
) -> Result<(), String> {
    restore_slot(work_dir, &bundle.kissconfig, 0)?;
    restore_slot(work_dir, &bundle.kissignore, 2)?;
    restore_slot(work_dir, &bundle.malvin_config, 3)?;
    restore_slot(work_dir, &bundle.gitignore, 4)?;
    restore_slot(work_dir, &bundle.malvin_config_workspace, 5)
}

#[cfg(test)]
#[path = "tests/slot_helpers.rs"]
mod slot_helpers;

#[cfg(test)]
mod tests;
