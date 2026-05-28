use std::path::Path;

use super::alloc;
use super::{KissConfigBackup, KissignoreBackup, MalvinChecksBackup, MalvinConfigBackup};

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissconfig_if_present(work_dir: &Path) -> Result<KissConfigBackup, String> {
    backup_workspace_kissconfig_if_present_with_id(work_dir, alloc::random_backup_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissconfig_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<KissConfigBackup, String> {
    super::backup_slot(0, work_dir, &mut generate_id)
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
    super::backup_slot(1, work_dir, &mut generate_id)
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
    super::backup_slot(2, work_dir, &mut generate_id)
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
    super::backup_slot(3, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_kissconfig_backup(
    work_dir: &Path,
    backup: &KissConfigBackup,
) -> Result<(), String> {
    super::restore_slot(work_dir, backup, 0)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_checks_backup(
    work_dir: &Path,
    backup: &MalvinChecksBackup,
) -> Result<(), String> {
    super::restore_slot(work_dir, backup, 1)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_kissignore_backup(
    work_dir: &Path,
    backup: &KissignoreBackup,
) -> Result<(), String> {
    super::restore_slot(work_dir, backup, 2)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_config_backup(
    work_dir: &Path,
    backup: &MalvinConfigBackup,
) -> Result<(), String> {
    super::restore_slot(work_dir, backup, 3)
}
