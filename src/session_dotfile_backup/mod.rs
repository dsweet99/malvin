mod alloc;
mod dotfile_backup_state;
pub(crate) mod gate_restore_checks;
pub(crate) mod gate_restore_merge;
pub(crate) mod gate_restore_repair;
mod gitignore_tree;
mod named_file_tree;
mod slots;
#[cfg(test)]
mod tree_test_support;
mod typed_slot_backups;
mod vision_tree;
mod wrappers;

pub use gate_restore_merge::{merge_and_sanitize_for_gate_restore, merge_for_gate_restore};
pub use gate_restore_repair::repair_invalid_malvin_home_config_on_disk;

use std::path::Path;

pub use dotfile_backup_state::{DotfileBackupPayload, DotfileBackupState, DotfileBackupStateRef};
pub use gitignore_tree::{GitignoreBackup, GitignoreFileBackup, GitignoreKind};
pub use named_file_tree::{NamedFile, NamedFileBackup, NamedFileKind};
pub use typed_slot_backups::{
    MalvinChecksBackup, MalvinChecksSlot, MalvinConfigWorkspaceBackup, MalvinConfigWorkspaceSlot,
    SlotBackup, SlotKind,
};
pub use vision_tree::{VisionBackup, VisionFileBackup, VisionKind};
pub use wrappers::{
    backup_workspace_malvin_checks_if_present, backup_workspace_malvin_checks_if_present_with_id,
    backup_workspace_malvin_config_workspace_if_present,
    backup_workspace_malvin_config_workspace_if_present_with_id,
    restore_workspace_malvin_checks_backup, restore_workspace_malvin_config_workspace_backup,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDotfileBackups {
    pub malvin_checks: MalvinChecksBackup,
    pub gitignore: GitignoreBackup,
    pub vision: VisionBackup,
    pub malvin_config_workspace: MalvinConfigWorkspaceBackup,
}

impl SessionDotfileBackups {
    #[must_use]
    pub const fn all_missing() -> Self {
        Self {
            malvin_checks: MalvinChecksBackup::Missing,
            gitignore: GitignoreBackup::Missing,
            vision: VisionBackup::Missing,
            malvin_config_workspace: MalvinConfigWorkspaceBackup::Missing,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn snapshot(work_dir: &Path) -> Result<Self, String> {
        Self::snapshot_with_id(work_dir, alloc::random_backup_id)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn snapshot_after_ensuring_home_config(work_dir: &Path) -> Result<Self, String> {
        repair_invalid_malvin_home_config_on_disk(work_dir)?;
        crate::malvin_config_file::ensure_malvin_config_file_if_missing(work_dir)?;
        Self::snapshot(work_dir)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn snapshot_with_id(
        work_dir: &Path,
        mut generate_id: impl FnMut(usize) -> String,
    ) -> Result<Self, String> {
        Ok(Self {
            malvin_checks: MalvinChecksBackup::backup(work_dir, &mut generate_id)?,
            gitignore: GitignoreBackup::backup(work_dir, &mut generate_id)?,
            vision: VisionBackup::backup(work_dir, &mut generate_id)?,
            malvin_config_workspace: MalvinConfigWorkspaceBackup::backup(
                work_dir,
                &mut generate_id,
            )?,
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn restore(&self, work_dir: &Path) -> Result<(), String> {
        restore_workspace_session_dotfiles(work_dir, self)
    }

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
    bundle
        .malvin_checks
        .restore(work_dir)
        .map(|()| crate::remove_legacy_malvin_checks_file(work_dir))
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_session_dotfiles_excluding_malvin_checks(
    work_dir: &Path,
    bundle: &SessionDotfileBackups,
) -> Result<(), String> {
    bundle.gitignore.restore(work_dir)?;
    bundle.vision.restore(work_dir)?;
    bundle.malvin_config_workspace.restore(work_dir)
}

#[cfg(test)]
#[path = "gate_restore_merge_kiss_cov_tests.rs"]
mod gate_restore_merge_kiss_cov_tests;
#[cfg(test)]
#[path = "gitignore_tree_kiss_cov_tests.rs"]
mod gitignore_tree_kiss_cov_tests;
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
#[path = "vision_tree_kiss_cov_tests.rs"]
mod vision_tree_kiss_cov_tests;
#[cfg(test)]
#[path = "wrappers_kiss_cov_tests.rs"]
mod wrappers_kiss_cov_tests;

#[cfg(test)]
#[path = "tests/slot_helpers.rs"]
mod slot_helpers;

#[cfg(test)]
mod tests;
