//! On-disk and in-memory repair for `kiss clamp` damage before gate-loop snapshots.

use std::path::Path;

use super::gate_restore_checks::is_bare_kiss_check_bytes;
use super::{DotfileBackupPayload, DotfileBackupState, SessionDotfileBackups};

fn dotfile_backup_state_from_path(path: &Path) -> Option<DotfileBackupState> {
    if !path.is_file() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    Some(DotfileBackupState::Present(DotfileBackupPayload {
        backup_path: path.to_path_buf(),
        bytes,
    }))
}

fn repair_invalid_malvin_checks_on_disk(work_dir: &Path) -> Result<(), String> {
    let path = crate::malvin_checks_path(work_dir);
    let Some(DotfileBackupState::Present(payload)) = dotfile_backup_state_from_path(&path) else {
        return crate::repo_gates::ensure_default_malvin_checks_file(work_dir);
    };
    if !is_bare_kiss_check_bytes(&payload.bytes) {
        return Ok(());
    }
    std::fs::remove_file(&path)
        .map_err(|e| format!("remove {}: {e}", path.display()))?;
    crate::repo_gates::ensure_default_malvin_checks_file(work_dir)
}

fn default_malvin_home_config_bytes() -> Result<Vec<u8>, String> {
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

fn malvin_home_config_bytes_need_repair(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return true;
    };
    text.parse::<toml::Value>().is_err()
}

fn repair_invalid_malvin_home_config_on_disk(work_dir: &Path) -> Result<(), String> {
    let path = crate::malvin_config_path(work_dir);
    if !path.is_file() {
        return Ok(());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if !malvin_home_config_bytes_need_repair(&bytes) {
        return Ok(());
    }
    std::fs::remove_file(&path).map_err(|e| format!("remove {}: {e}", path.display()))?;
    crate::malvin_config_file::ensure_malvin_config_file_if_missing(work_dir)
}

fn default_malvin_checks_bytes(work_dir: &Path) -> Vec<u8> {
    let lines = crate::repo_gates::builtin_gate_command_lines(work_dir);
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    content.into_bytes()
}

fn repair_malvin_checks_bytes(work_dir: &Path, bytes: &[u8]) -> Option<Vec<u8>> {
    is_bare_kiss_check_bytes(bytes).then(|| default_malvin_checks_bytes(work_dir))
}

fn sanitize_malvin_checks_slot(work_dir: &Path, slot: &mut DotfileBackupState) {
    let DotfileBackupState::Present(payload) = slot else {
        return;
    };
    if let Some(fixed) = repair_malvin_checks_bytes(work_dir, &payload.bytes) {
        payload.bytes = fixed;
    }
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

/// Sanitize known `kiss clamp` damage inside a carry-forward backup bundle.
///
/// Disk repair alone cannot fix poisoned bytes held in the parent gate loop's
/// in-memory `SessionDotfileBackups` when merge leaves both anchor and progress corrupt.
pub fn sanitize_clamp_damaged_dotfiles_in_bundle(
    bundle: &mut SessionDotfileBackups,
    work_dir: &Path,
) {
    sanitize_malvin_checks_slot(work_dir, &mut bundle.malvin_checks);
    sanitize_malvin_config_slot(&mut bundle.malvin_config);
}

/// Repair known `kiss clamp` damage on disk before gate-loop snapshots.
///
/// Removes bare `kiss` checks (replacing with repo defaults). Does not rewrite
/// `gate.test_coverage_threshold`; kiss clamp sets that naturally when needed.
pub fn repair_clamp_damaged_dotfiles_on_disk(work_dir: &Path) -> Result<(), String> {
    repair_invalid_malvin_home_config_on_disk(work_dir)?;
    repair_invalid_malvin_checks_on_disk(work_dir)
}
#[cfg(test)]
#[path = "gate_restore_repair_test.rs"]
mod gate_restore_repair_test;
