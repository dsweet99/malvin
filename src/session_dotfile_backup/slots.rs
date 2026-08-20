use std::path::{Path, PathBuf};

use super::DotfileBackupState;
use super::alloc::{DotfileBackupLabels, allocate_backup_dir, remove_if_exists};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct DotfileSpecRow {
    pub rel: &'static str,
    pub home_subdir: &'static str,
    pub mkdir_lbl: &'static str,
    pub collision_lbl: &'static str,
    pub restore_lbl: &'static str,
    pub copy_err: &'static str,
    pub restore_copy_err: &'static str,
}

impl DotfileSpecRow {
    pub(super) const fn rel_path(self) -> &'static str {
        self.rel
    }
}

const fn labels(spec: &DotfileSpecRow) -> DotfileBackupLabels {
    DotfileBackupLabels {
        mkdir: spec.mkdir_lbl,
        collision: spec.collision_lbl,
        restore: spec.restore_lbl,
    }
}

pub(super) fn dotfile_source_path(slot: usize, work_dir: &Path) -> PathBuf {
    if DOTFILE_ROWS[slot].rel == crate::MALVIN_CHECKS_REL {
        crate::resolve_malvin_checks_path(work_dir)
    } else {
        work_dir.join(DOTFILE_ROWS[slot].rel)
    }
}

const GITIGNORE_FILE: &str = ".gitignore";
pub(super) const MALVIN_CONFIG_WORKSPACE_SLOT: usize = 2;

pub(super) const DOTFILE_ROWS: [DotfileSpecRow; 3] = [
    DotfileSpecRow {
        rel: crate::MALVIN_CHECKS_REL,
        home_subdir: "malvin_checks",
        mkdir_lbl: "malvin_checks backup mkdir",
        collision_lbl: "malvin_checks backup mkdir",
        restore_lbl: "malvin_checks restore",
        copy_err: ".malvin/gates backup copy",
        restore_copy_err: "malvin_checks restore",
    },
    DotfileSpecRow {
        rel: GITIGNORE_FILE,
        home_subdir: "gitignore",
        mkdir_lbl: "gitignore backup mkdir",
        collision_lbl: "gitignore backup mkdir",
        restore_lbl: "gitignore restore",
        copy_err: ".gitignore backup copy",
        restore_copy_err: "gitignore restore",
    },
    DotfileSpecRow {
        rel: crate::MALVIN_CONFIG_REL,
        home_subdir: "malvin_config_workspace",
        mkdir_lbl: "malvin_config_workspace backup mkdir",
        collision_lbl: "malvin_config_workspace backup mkdir",
        restore_lbl: "malvin_config_workspace restore",
        copy_err: "workspace .malvin/config.toml backup copy",
        restore_copy_err: "malvin_config_workspace restore",
    },
];

pub(super) fn backup_slot(
    slot: usize,
    work_dir: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
) -> Result<DotfileBackupState, String> {
    let spec = &DOTFILE_ROWS[slot];
    let _ = spec.rel_path();
    let src = dotfile_source_path(slot, work_dir);
    if !src.is_file() {
        return Ok(DotfileBackupState::Missing);
    }
    let root = crate::workspace_paths::snapshot_category_dir(spec.home_subdir);
    let lbls = labels(spec);
    let dest_dir = allocate_backup_dir(&root, generate_id, &lbls)?;
    let dest_file = dest_dir.join(spec.rel);
    if let Some(parent) = dest_file.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", spec.mkdir_lbl))?;
    }
    let bytes = std::fs::read(&src).map_err(|e| format!("{}: {e}", spec.copy_err))?;
    if let Err(e) = std::fs::write(&dest_file, &bytes) {
        let _ = std::fs::remove_dir_all(&dest_dir);
        return Err(format!("{}: {e}", spec.copy_err));
    }
    Ok(DotfileBackupState::Present(super::DotfileBackupPayload {
        backup_path: dest_file,
        bytes,
    }))
}

pub(super) fn restore_slot(
    work_dir: &Path,
    backup: &DotfileBackupState,
    slot: usize,
) -> Result<(), String> {
    let spec = &DOTFILE_ROWS[slot];
    let _ = spec.rel_path();
    let dst = dotfile_source_path(slot, work_dir);
    let lbls = labels(spec);
    match backup {
        DotfileBackupState::Missing => remove_if_exists(&dst, lbls.restore),
        DotfileBackupState::Present(payload) => {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("{}: {e}", spec.restore_lbl))?;
            }
            std::fs::write(&dst, &payload.bytes)
                .map_err(|e| format!("{}: {e}", spec.restore_copy_err))
        }
    }
}

#[cfg(test)]
pub(super) const fn labels_for_test(row: &DotfileSpecRow) -> DotfileBackupLabels {
    labels(row)
}
