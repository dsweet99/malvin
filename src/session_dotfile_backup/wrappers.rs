use std::path::Path;

use super::alloc;
use super::{KissConfigBackup, KissignoreBackup, MalvinChecksBackup, MalvinConfigBackup, MalvinConfigWorkspaceBackup};

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissconfig_if_present(work_dir: &Path) -> Result<KissConfigBackup, String> {
    backup_workspace_kissconfig_if_present_with_id(work_dir, alloc::random_backup_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissconfig_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<KissConfigBackup, String> {
    super::slots::backup_slot(0, work_dir, &mut generate_id)
}

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
    super::slots::backup_slot(1, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissignore_if_present(work_dir: &Path) -> Result<KissignoreBackup, String> {
    backup_workspace_kissignore_if_present_with_id(work_dir, alloc::random_backup_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissignore_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<KissignoreBackup, String> {
    super::slots::backup_slot(2, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_config_if_present(
    work_dir: &Path,
) -> Result<MalvinConfigBackup, String> {
    backup_workspace_malvin_config_if_present_with_id(work_dir, alloc::random_backup_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_config_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<MalvinConfigBackup, String> {
    super::slots::backup_slot(3, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_kissconfig_backup(
    work_dir: &Path,
    backup: &KissConfigBackup,
) -> Result<(), String> {
    super::slots::restore_slot(work_dir, backup, 0)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_checks_backup(
    work_dir: &Path,
    backup: &MalvinChecksBackup,
) -> Result<(), String> {
    super::slots::restore_slot(work_dir, backup, 1)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_kissignore_backup(
    work_dir: &Path,
    backup: &KissignoreBackup,
) -> Result<(), String> {
    super::slots::restore_slot(work_dir, backup, 2)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_config_backup(
    work_dir: &Path,
    backup: &MalvinConfigBackup,
) -> Result<(), String> {
    super::slots::restore_slot(work_dir, backup, 3)
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
    super::slots::backup_slot(5, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_config_workspace_backup(
    work_dir: &Path,
    backup: &MalvinConfigWorkspaceBackup,
) -> Result<(), String> {
    super::slots::restore_slot(work_dir, backup, 5)
}
#[cfg(test)]
#[path = "wrappers_test.rs"]
mod wrappers_test;
#[cfg(test)]
#[path = "wrappers_kiss_cov_test.rs"]
mod wrappers_kiss_cov_test;
