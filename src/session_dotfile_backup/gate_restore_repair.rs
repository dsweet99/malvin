//! On-disk and in-memory repair for invalid session dotfiles before gate-loop snapshots.

use std::path::Path;

use super::{DotfileBackupState, SessionDotfileBackups};

pub(crate) fn default_malvin_home_config_bytes() -> Result<Vec<u8>, String> {
    let template = crate::malvin_config_file::parse_template_value()?;
    let mut value = toml::Value::Table(toml::map::Map::new());
    crate::malvin_config_file::merge_missing_keys(&mut value, &template);
    let mut text = toml::to_string_pretty(&value)
        .map_err(|e| format!("serialize default home config: {e}"))?;
    if !text.ends_with('\n') {
        text.push('\n');
    }
    Ok(text.into_bytes())
}

/// Session restore must not re-materialize a 0-byte home config (sticky empty file).
pub(crate) fn bytes_for_malvin_home_config_restore(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.is_empty() {
        default_malvin_home_config_bytes()
    } else {
        Ok(bytes.to_vec())
    }
}

fn malvin_home_config_bytes_need_repair(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return true;
    };
    text.parse::<toml::Value>().is_err()
}

fn repair_malvin_home_config_on_disk_impl(work_dir: &Path) -> Result<(), String> {
    if !crate::workspace_paths::home_malvin_config_delete_allowed() {
        return Ok(());
    }
    let path = crate::malvin_config_path(work_dir);
    if !path.is_file() {
        return Ok(());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if !malvin_home_config_bytes_need_repair(&bytes) {
        return Ok(());
    }
    super::alloc::remove_if_exists(&path, "malvin home config repair")
        .map_err(|e| format!("remove {}: {e}", path.display()))?;
    crate::malvin_config_file::ensure_malvin_config_file_if_missing(work_dir)
}

fn sanitize_malvin_config_slot(slot: &mut DotfileBackupState) {
    let DotfileBackupState::Present(payload) = slot else {
        return;
    };
    if !malvin_home_config_bytes_need_repair(&payload.bytes) {
        return;
    }
    if let Ok(fixed) = default_malvin_home_config_bytes() {
        payload.bytes = fixed;
    }
}

/// Sanitize invalid home config bytes inside a carry-forward backup bundle.
pub fn sanitize_invalid_malvin_home_config_in_bundle(
    bundle: &mut SessionDotfileBackups,
    work_dir: &Path,
) {
    let _ = work_dir;
    sanitize_malvin_config_slot(&mut bundle.malvin_config);
}

/// Repair invalid `~/.malvin_home/config.toml` on disk before gate-loop snapshots.
pub fn repair_invalid_malvin_home_config_on_disk(work_dir: &Path) -> Result<(), String> {
    repair_malvin_home_config_on_disk_impl(work_dir)
}

#[cfg(test)]
#[path = "gate_restore_repair_tests.rs"]
mod gate_restore_repair_tests;
