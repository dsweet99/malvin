mod alloc;
mod wrappers;

use std::path::{Path, PathBuf};

pub(crate) use alloc::{allocate_backup_dir, malvin_home_dir, remove_if_exists, DotfileBackupLabels};
pub use wrappers::{
    backup_workspace_kissconfig_if_present, backup_workspace_kissconfig_if_present_with_id,
    backup_workspace_kissignore_if_present, backup_workspace_kissignore_if_present_with_id,
    backup_workspace_malvin_checks_if_present, backup_workspace_malvin_checks_if_present_with_id,
    backup_workspace_malvin_config_if_present, backup_workspace_malvin_config_if_present_with_id,
    restore_workspace_kissconfig_backup, restore_workspace_kissignore_backup,
    restore_workspace_malvin_checks_backup, restore_workspace_malvin_config_backup,
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

const KISSCONFIG_FILE: &str = ".kissconfig";
const KISSIGNORE_FILE: &str = ".kissignore";

const DOTFILE_ROWS: [DotfileSpecRow; 4] = [
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDotfileBackups {
    pub kissconfig: KissConfigBackup,
    pub malvin_checks: MalvinChecksBackup,
    pub kissignore: KissignoreBackup,
    pub malvin_config: MalvinConfigBackup,
}

pub(super) fn backup_slot(
    slot: usize,
    work_dir: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
) -> Result<DotfileBackupState, String> {
    let spec = &DOTFILE_ROWS[slot];
    let src = work_dir.join(spec.rel);
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
    let dst = work_dir.join(spec.rel);
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
    pub const fn from_parts(
        kissconfig: KissConfigBackup,
        malvin_checks: MalvinChecksBackup,
        kissignore: KissignoreBackup,
        malvin_config: MalvinConfigBackup,
    ) -> Self {
        Self {
            kissconfig,
            malvin_checks,
            kissignore,
            malvin_config,
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
    restore_slot(work_dir, &bundle.kissconfig, 0)?;
    restore_slot(work_dir, &bundle.malvin_checks, 1)?;
    restore_slot(work_dir, &bundle.kissignore, 2)?;
    restore_slot(work_dir, &bundle.malvin_config, 3)
        .map(|()| crate::remove_legacy_malvin_checks_file(work_dir))
}

#[cfg(test)]
mod slot_helpers {
    use super::*;

    #[test]
    fn dotfile_slot_helpers_and_session_restore_noop() {
        let _ = labels(&DOTFILE_ROWS[0]);
        let tmp = tempfile::tempdir().unwrap();
        let mut id = |n: usize| format!("slot{n}");
        let _ = backup_slot(0, tmp.path(), &mut id);
        let _ = restore_slot(tmp.path(), &DotfileBackupState::Missing, 1);
        let bundle = SessionDotfileBackups::from_parts(
            DotfileBackupState::Missing,
            DotfileBackupState::Missing,
            DotfileBackupState::Missing,
            DotfileBackupState::Missing,
        );
        restore_workspace_session_dotfiles(tmp.path(), &bundle).unwrap();
    }
}

#[cfg(test)]
mod tests;
