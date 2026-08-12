use std::path::Path;

use super::alloc;
use super::{MalvinChecksBackup, MalvinConfigWorkspaceBackup};

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_checks_if_present(
    work_dir: &Path,
) -> Result<MalvinChecksBackup, String> {
    backup_workspace_malvin_checks_if_present_with_id(work_dir, alloc::random_backup_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_checks_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<MalvinChecksBackup, String> {
    super::slots::backup_slot(0, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_checks_backup(
    work_dir: &Path,
    backup: &MalvinChecksBackup,
) -> Result<(), String> {
    super::slots::restore_slot(work_dir, backup, 0)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_config_workspace_if_present(
    work_dir: &Path,
) -> Result<MalvinConfigWorkspaceBackup, String> {
    backup_workspace_malvin_config_workspace_if_present_with_id(work_dir, super::alloc::random_backup_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_config_workspace_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<MalvinConfigWorkspaceBackup, String> {
    super::slots::backup_slot(super::slots::MALVIN_CONFIG_WORKSPACE_SLOT, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_config_workspace_backup(
    work_dir: &Path,
    backup: &MalvinConfigWorkspaceBackup,
) -> Result<(), String> {
    super::slots::restore_slot(work_dir, backup, super::slots::MALVIN_CONFIG_WORKSPACE_SLOT)
}
