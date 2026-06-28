use std::path::{Path, PathBuf};

use super::alloc::{allocate_backup_dir, remove_if_exists, DotfileBackupLabels};
use crate::workspace_paths::{snapshot_category_dir, MALVIN_HOME_CONFIG_FILE};
use super::DotfileBackupState;

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
    if slot == 3 {
        crate::malvin_config_path(work_dir)
    } else if DOTFILE_ROWS[slot].rel == crate::MALVIN_CHECKS_REL {
        crate::resolve_malvin_checks_path(work_dir)
    } else {
        work_dir.join(DOTFILE_ROWS[slot].rel)
    }
}

const KISSCONFIG_FILE: &str = ".kissconfig";
const KISSIGNORE_FILE: &str = ".kissignore";
const GITIGNORE_FILE: &str = ".gitignore";
const MALVIN_CONFIG_SLOT: usize = 3;

pub(super) const DOTFILE_ROWS: [DotfileSpecRow; 6] = [
    DotfileSpecRow {
        rel: KISSCONFIG_FILE,
        home_subdir: "kissconfig",
        mkdir_lbl: "kissconfig backup mkdir",
        collision_lbl: "kissconfig backup mkdir",
        restore_lbl: "kissconfig restore",
        copy_err: ".kissconfig backup copy",
        restore_copy_err: "kissconfig restore",
    },
    DotfileSpecRow {
        rel: crate::MALVIN_CHECKS_REL,
        home_subdir: "malvin_checks",
        mkdir_lbl: "malvin_checks backup mkdir",
        collision_lbl: "malvin_checks backup mkdir",
        restore_lbl: "malvin_checks restore",
        copy_err: ".malvin/checks backup copy",
        restore_copy_err: "malvin_checks restore",
    },
    DotfileSpecRow {
        rel: KISSIGNORE_FILE,
        home_subdir: "kissignore",
        mkdir_lbl: "kissignore backup mkdir",
        collision_lbl: "kissignore backup mkdir",
        restore_lbl: "kissignore restore",
        copy_err: ".kissignore backup copy",
        restore_copy_err: "kissignore restore",
    },
    DotfileSpecRow {
        rel: MALVIN_HOME_CONFIG_FILE,
        home_subdir: "malvin_config",
        mkdir_lbl: "malvin_config backup mkdir",
        collision_lbl: "malvin_config backup mkdir",
        restore_lbl: "malvin_config restore",
        copy_err: "~/.malvin_home/config.toml backup copy",
        restore_copy_err: "malvin_config restore",
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
    let root = snapshot_category_dir(spec.home_subdir);
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

fn is_ensured_default_malvin_config(bytes: &[u8]) -> bool {
    let Ok(template) = crate::malvin_config_file::parse_template_value() else {
        return false;
    };
    let mut ensured = toml::Value::Table(toml::map::Map::new());
    crate::malvin_config_file::merge_missing_keys(&mut ensured, &template);
    let Ok(mut ensured_text) = toml::to_string_pretty(&ensured) else {
        return false;
    };
    if !ensured_text.ends_with('\n') {
        ensured_text.push('\n');
    }
    std::str::from_utf8(bytes).is_ok_and(|on_disk| on_disk == ensured_text)
}

fn restore_malvin_config_missing(dst: &Path, lbls: &DotfileBackupLabels) -> Result<(), String> {
    if !crate::workspace_paths::home_malvin_config_delete_allowed() {
        return Ok(());
    }
    if !dst.exists() {
        return Ok(());
    }
    if !dst.is_file() {
        return remove_if_exists(dst, lbls.restore);
    }
    let bytes = std::fs::read(dst).map_err(|e| format!("{}: {e}", lbls.restore))?;
    if is_ensured_default_malvin_config(&bytes) {
        return remove_if_exists(dst, lbls.restore);
    }
    let keep = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|text| text.parse::<toml::Value>().ok())
        .is_some();
    if keep {
        Ok(())
    } else {
        remove_if_exists(dst, lbls.restore)
    }
}

pub(super) fn restore_slot(work_dir: &Path, backup: &DotfileBackupState, slot: usize) -> Result<(), String> {
    let spec = &DOTFILE_ROWS[slot];
    let _ = spec.rel_path();
    let dst = dotfile_source_path(slot, work_dir);
    let lbls = labels(spec);
    match backup {
        DotfileBackupState::Missing => {
            if slot == MALVIN_CONFIG_SLOT {
                restore_malvin_config_missing(&dst, &lbls)
            } else {
                remove_if_exists(&dst, lbls.restore)
            }
        }
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

#[cfg(test)]
pub(super) fn restore_malvin_config_missing_for_test(
    dst: &Path,
    lbls: &DotfileBackupLabels,
) -> Result<(), String> {
    restore_malvin_config_missing(dst, lbls)
}
