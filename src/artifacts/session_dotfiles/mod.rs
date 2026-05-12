use std::path::{Path, PathBuf};

use super::dotfile_backup::{
    DotfileBackupLabels, allocate_backup_dir, remove_if_exists,
};
use super::run_id::random_alnum;

struct DotfileSpecRow {
    rel: &'static str,
    home_subdir: &'static str,
    mkdir_lbl: &'static str,
    collision_lbl: &'static str,
    restore_lbl: &'static str,
    copy_err: &'static str,
    restore_copy_err: &'static str,
}

const fn labels(spec: &DotfileSpecRow) -> DotfileBackupLabels {
    DotfileBackupLabels {
        mkdir: spec.mkdir_lbl,
        collision: spec.collision_lbl,
        restore: spec.restore_lbl,
    }
}

const DOTFILE_ROWS: [DotfileSpecRow; 3] = [
    DotfileSpecRow {
        rel: crate::repo_gates::KISSCONFIG_FILE,
        home_subdir: "kissconfigs",
        mkdir_lbl: "kissconfig backup mkdir",
        collision_lbl: "kissconfig backup mkdir",
        restore_lbl: "kissconfig restore",
        copy_err: ".kissconfig backup copy",
        restore_copy_err: "kissconfig restore",
    },
    DotfileSpecRow {
        rel: crate::repo_gates::MALVIN_CHECKS_FILE,
        home_subdir: "malvin_checks_snapshots",
        mkdir_lbl: "malvin_checks backup mkdir",
        collision_lbl: "malvin_checks backup mkdir",
        restore_lbl: "malvin_checks restore",
        copy_err: ".malvin_checks backup copy",
        restore_copy_err: "malvin_checks restore",
    },
    DotfileSpecRow {
        rel: crate::repo_gates::KISSIGNORE_FILE,
        home_subdir: "kissignore_snapshots",
        mkdir_lbl: "kissignore backup mkdir",
        collision_lbl: "kissignore backup mkdir",
        restore_lbl: "kissignore restore",
        copy_err: ".kissignore backup copy",
        restore_copy_err: "kissignore restore",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DotfileBackupState {
    Missing,
    Present(PathBuf),
}

pub type KissConfigBackup = DotfileBackupState;
pub type MalvinChecksBackup = DotfileBackupState;
pub type KissignoreBackup = DotfileBackupState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDotfileBackups {
    pub kissconfig: KissConfigBackup,
    pub malvin_checks: MalvinChecksBackup,
    pub kissignore: KissignoreBackup,
}

fn backup_slot(
    slot: usize,
    work_dir: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
) -> Result<DotfileBackupState, String> {
    let spec = &DOTFILE_ROWS[slot];
    let src = work_dir.join(spec.rel);
    if !src.is_file() {
        return Ok(DotfileBackupState::Missing);
    }
    let root = crate::prompts::user_home_dir()
        .join(".malvin")
        .join(spec.home_subdir);
    let lbls = labels(spec);
    let dest_dir = allocate_backup_dir(&root, generate_id, &lbls)?;
    let dest_file = dest_dir.join(spec.rel);
    std::fs::copy(&src, &dest_file).map_err(|e| format!("{}: {e}", spec.copy_err))?;
    Ok(DotfileBackupState::Present(dest_file))
}

fn restore_slot(
    work_dir: &Path,
    backup: &DotfileBackupState,
    slot: usize,
) -> Result<(), String> {
    let spec = &DOTFILE_ROWS[slot];
    let dst = work_dir.join(spec.rel);
    let lbls = labels(spec);
    match backup {
        DotfileBackupState::Missing => remove_if_exists(&dst, lbls.restore),
        DotfileBackupState::Present(backup_path) => std::fs::copy(backup_path, &dst)
            .map_err(|e| format!("{}: {e}", spec.restore_copy_err))
            .map(|_| ()),
    }
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
    backup_slot(0, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_checks_if_present(
    work_dir: &Path,
) -> Result<MalvinChecksBackup, String> {
    backup_workspace_malvin_checks_if_present_with_id(work_dir, |_| random_alnum(5))
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_malvin_checks_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<MalvinChecksBackup, String> {
    backup_slot(1, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissignore_if_present(work_dir: &Path) -> Result<KissignoreBackup, String> {
    backup_workspace_kissignore_if_present_with_id(work_dir, |_| random_alnum(5))
}

#[allow(clippy::missing_errors_doc)]
pub fn backup_workspace_kissignore_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<KissignoreBackup, String> {
    backup_slot(2, work_dir, &mut generate_id)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_kissconfig_backup(
    work_dir: &Path,
    backup: &KissConfigBackup,
) -> Result<(), String> {
    restore_slot(work_dir, backup, 0)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_malvin_checks_backup(
    work_dir: &Path,
    backup: &MalvinChecksBackup,
) -> Result<(), String> {
    restore_slot(work_dir, backup, 1)
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_kissignore_backup(
    work_dir: &Path,
    backup: &KissignoreBackup,
) -> Result<(), String> {
    restore_slot(work_dir, backup, 2)
}

impl SessionDotfileBackups {
    #[must_use]
    pub const fn from_parts(
        kissconfig: KissConfigBackup,
        malvin_checks: MalvinChecksBackup,
        kissignore: KissignoreBackup,
    ) -> Self {
        Self {
            kissconfig,
            malvin_checks,
            kissignore,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn snapshot(work_dir: &Path) -> Result<Self, String> {
        Self::snapshot_with_id(work_dir, |_| random_alnum(5))
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
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn restore(&self, work_dir: &Path) -> Result<(), String> {
        restore_workspace_session_dotfiles(work_dir, self)
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn restore_workspace_session_dotfiles(
    work_dir: &Path,
    bundle: &SessionDotfileBackups,
) -> Result<(), String> {
    let k = restore_slot(work_dir, &bundle.kissconfig, 0);
    let m = restore_slot(work_dir, &bundle.malvin_checks, 1);
    let i = restore_slot(work_dir, &bundle.kissignore, 2);
    let mut errs = Vec::new();
    if let Err(e) = k {
        errs.push(e);
    }
    if let Err(e) = m {
        errs.push(e);
    }
    if let Err(e) = i {
        errs.push(e);
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs.join("; "))
    }
}

#[cfg(test)]
mod tests;
