mod alloc;
pub(crate) mod gate_restore_checks;
pub(crate) mod gate_restore_merge;
pub(crate) mod gate_restore_repair;
mod gitignore_tree;
mod slots;
mod wrappers;

pub use gate_restore_merge::{merge_and_sanitize_for_gate_restore, merge_for_gate_restore};
pub use gate_restore_repair::{
    repair_clamp_damaged_dotfiles_on_disk, sanitize_clamp_damaged_dotfiles_in_bundle,
};

use std::path::Path;

pub use gitignore_tree::{
    backup_workspace_gitignore_if_present, backup_workspace_gitignore_if_present_with_id,
    restore_workspace_gitignore_backup, GitignoreBackup, GitignoreFileBackup,
};
pub use wrappers::{
    backup_workspace_kissconfig_if_present, backup_workspace_kissconfig_if_present_with_id,
    backup_workspace_kissignore_if_present, backup_workspace_kissignore_if_present_with_id,
    backup_workspace_malvin_checks_if_present, backup_workspace_malvin_checks_if_present_with_id,
    backup_workspace_malvin_config_if_present, backup_workspace_malvin_config_if_present_with_id,
    backup_workspace_malvin_config_workspace_if_present,
    backup_workspace_malvin_config_workspace_if_present_with_id,
    restore_workspace_kissconfig_backup, restore_workspace_kissignore_backup,
    restore_workspace_malvin_checks_backup, restore_workspace_malvin_config_backup,
    restore_workspace_malvin_config_workspace_backup,
};

use slots::{backup_slot, restore_slot};

/// Captured dotfile bytes at snapshot time plus the historical disk location under `~/.malvin_home`.
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

    /// Like [`snapshot`], but ensures `~/.malvin_home/config.toml` exists first.
    ///
    /// Gate workflows (`code`, `tidy`, …) materialize home config at CLI entry; without this,
    /// a prior restore with [`DotfileBackupState::Missing`] can delete the file and the next
    /// snapshot records `Missing` again, so every later restore keeps removing it.
    #[allow(clippy::missing_errors_doc)]
    pub fn snapshot_after_ensuring_home_config(work_dir: &Path) -> Result<Self, String> {
        crate::malvin_config_file::ensure_malvin_config_file_if_missing(work_dir)?;
        Self::snapshot(work_dir)
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
            gitignore: gitignore_tree::backup_gitignore_tree(work_dir, &mut generate_id)?,
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
    gitignore_tree::restore_workspace_gitignore_backup(work_dir, &bundle.gitignore)?;
    restore_slot(work_dir, &bundle.malvin_config_workspace, 5)
}

#[cfg(test)]
#[path = "mod_kiss_cov_tests.rs"]
mod mod_kiss_cov_tests;
#[cfg(test)]
mod slots_kiss_cov_shared;
#[cfg(test)]
#[path = "slots_kiss_cov_tests.rs"]
mod slots_kiss_cov_tests;
#[cfg(test)]
#[path = "slots_kiss_cov_tests_b.rs"]
mod slots_kiss_cov_tests_b;

#[cfg(test)]
#[path = "tests/slot_helpers.rs"]
mod slot_helpers;

#[cfg(test)]
mod tests;
