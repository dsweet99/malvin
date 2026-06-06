use std::path::{Path, PathBuf};

use super::alloc::{allocate_backup_dir, malvin_home_dir, remove_if_exists, DotfileBackupLabels};
use super::DotfileBackupState;

pub(super) struct DotfileSpecRow {
    pub rel: &'static str,
    pub home_subdir: &'static str,
    pub mkdir_lbl: &'static str,
    pub collision_lbl: &'static str,
    pub restore_lbl: &'static str,
    pub copy_err: &'static str,
    pub restore_copy_err: &'static str,
}

const fn labels(spec: &DotfileSpecRow) -> DotfileBackupLabels {
    DotfileBackupLabels {
        mkdir: spec.mkdir_lbl,
        collision: spec.collision_lbl,
        restore: spec.restore_lbl,
    }
}

pub(super) fn dotfile_source_path(slot: usize, work_dir: &Path) -> PathBuf {
    if slot == 3 {
        crate::malvin_config_path(work_dir)
    } else {
        work_dir.join(DOTFILE_ROWS[slot].rel)
    }
}

const KISSCONFIG_FILE: &str = ".kissconfig";
const KISSIGNORE_FILE: &str = ".kissignore";
const GITIGNORE_FILE: &str = ".gitignore";

pub(super) const DOTFILE_ROWS: [DotfileSpecRow; 6] = [
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
    DotfileSpecRow {
        rel: crate::MALVIN_CONFIG_REL,
        home_subdir: "malvin_config_workspace_snapshots",
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

pub(super) fn restore_slot(work_dir: &Path, backup: &DotfileBackupState, slot: usize) -> Result<(), String> {
    let spec = &DOTFILE_ROWS[slot];
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
