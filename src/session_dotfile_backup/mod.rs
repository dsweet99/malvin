mod alloc;
mod wrappers;

use std::path::{Path, PathBuf};

pub(crate) use alloc::{allocate_backup_dir, malvin_home_dir, remove_if_exists, DotfileBackupLabels};
pub use wrappers::{
    backup_workspace_gitignore_if_present, backup_workspace_gitignore_if_present_with_id,
    backup_workspace_kissconfig_if_present, backup_workspace_kissconfig_if_present_with_id,
    backup_workspace_kissignore_if_present, backup_workspace_kissignore_if_present_with_id,
    backup_workspace_malvin_checks_if_present, backup_workspace_malvin_checks_if_present_with_id,
    backup_workspace_malvin_config_if_present, backup_workspace_malvin_config_if_present_with_id,
    restore_workspace_gitignore_backup, restore_workspace_kissconfig_backup,
    restore_workspace_kissignore_backup, restore_workspace_malvin_checks_backup,
    restore_workspace_malvin_config_backup,
};

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

fn dotfile_source_path(slot: usize, work_dir: &Path) -> PathBuf {
    if slot == 3 {
        crate::malvin_config_path(work_dir)
    } else {
        work_dir.join(DOTFILE_ROWS[slot].rel)
    }
}

const KISSCONFIG_FILE: &str = ".kissconfig";
const KISSIGNORE_FILE: &str = ".kissignore";
const GITIGNORE_FILE: &str = ".gitignore";

const DOTFILE_ROWS: [DotfileSpecRow; 5] = [
    DotfileSpecRow {
        rel: KISSCONFIG_FILE,
        home_subdir: "kissconfigs",
        mkdir_lbl: "kissconfig backup mkdir",
        collision_lbl: "kissconfig backup mkdir",
        restore_lbl: "kissconfig restore",
        copy_err: ".kissconfig backup copy",
        restore_copy_err: "kissconfig restore",
    },
    DotfileSpecRow {
        rel: crate::MALVIN_CHECKS_REL,
        home_subdir: "malvin_checks_snapshots",
        mkdir_lbl: "malvin_checks backup mkdir",
        collision_lbl: "malvin_checks backup mkdir",
        restore_lbl: "malvin_checks restore",
        copy_err: ".malvin/checks backup copy",
        restore_copy_err: "malvin_checks restore",
    },
    DotfileSpecRow {
        rel: KISSIGNORE_FILE,
        home_subdir: "kissignore_snapshots",
        mkdir_lbl: "kissignore backup mkdir",
        collision_lbl: "kissignore backup mkdir",
        restore_lbl: "kissignore restore",
        copy_err: ".kissignore backup copy",
        restore_copy_err: "kissignore restore",
    },
    DotfileSpecRow {
        rel: crate::MALVIN_CONFIG_REL,
        home_subdir: "malvin_config_snapshots",
        mkdir_lbl: "malvin_config backup mkdir",
        collision_lbl: "malvin_config backup mkdir",
        restore_lbl: "malvin_config restore",
        copy_err: ".malvin/config.toml backup copy",
        restore_copy_err: "malvin_config restore",
    },
    DotfileSpecRow {
        rel: GITIGNORE_FILE,
        home_subdir: "gitignore_snapshots",
        mkdir_lbl: "gitignore backup mkdir",
        collision_lbl: "gitignore backup mkdir",
        restore_lbl: "gitignore restore",
        copy_err: ".gitignore backup copy",
        restore_copy_err: "gitignore restore",
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
pub type MalvinConfigBackup = DotfileBackupState;
pub type GitignoreBackup = DotfileBackupState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDotfileParts {
    pub kissconfig: KissConfigBackup,
    pub malvin_checks: MalvinChecksBackup,
    pub kissignore: KissignoreBackup,
    pub malvin_config: MalvinConfigBackup,
    pub gitignore: GitignoreBackup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDotfileBackups {
    pub kissconfig: KissConfigBackup,
    pub malvin_checks: MalvinChecksBackup,
    pub kissignore: KissignoreBackup,
    pub malvin_config: MalvinConfigBackup,
    pub gitignore: GitignoreBackup,
}

pub(super) fn backup_slot(
    slot: usize,
    work_dir: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
) -> Result<DotfileBackupState, String> {
    let spec = &DOTFILE_ROWS[slot];
    let src = dotfile_source_path(slot, work_dir);
    if !src.is_file() {
        return Ok(DotfileBackupState::Missing);
    }
    let root = malvin_home_dir().join(".malvin").join(spec.home_subdir);
    let lbls = labels(spec);
    let dest_dir = allocate_backup_dir(&root, generate_id, &lbls)?;
    let dest_file = dest_dir.join(spec.rel);
    if let Some(parent) = dest_file.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", spec.mkdir_lbl))?;
    }
    if let Err(e) = std::fs::copy(&src, &dest_file) {
        let _ = std::fs::remove_dir_all(&dest_dir);
        return Err(format!("{}: {e}", spec.copy_err));
    }
    Ok(DotfileBackupState::Present(dest_file))
}

pub(super) fn restore_slot(work_dir: &Path, backup: &DotfileBackupState, slot: usize) -> Result<(), String> {
    let spec = &DOTFILE_ROWS[slot];
    let dst = dotfile_source_path(slot, work_dir);
    let lbls = labels(spec);
    match backup {
        DotfileBackupState::Missing => remove_if_exists(&dst, lbls.restore),
        DotfileBackupState::Present(backup_path) => {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("{}: {e}", spec.restore_lbl))?;
            }
            std::fs::copy(backup_path, &dst)
                .map_err(|e| format!("{}: {e}", spec.restore_copy_err))
                .map(|_| ())
        }
    }
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
    restore_slot(work_dir, &bundle.gitignore, 4)
}

#[cfg(test)]
#[path = "tests/slot_helpers.rs"]
mod slot_helpers;

#[cfg(test)]
mod tests;
