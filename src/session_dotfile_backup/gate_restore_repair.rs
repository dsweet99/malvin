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

fn kissconfig_value_needing_threshold_repair(bytes: &[u8]) -> Option<toml::Value> {
    let text = std::str::from_utf8(bytes).ok()?;
    let value: toml::Value = text.parse().ok()?;
    crate::repo_checks::kissconfig_warn::should_warn_low_test_coverage(&value)
        .then_some(value)
}

fn write_toml_pretty(path: &Path, value: &toml::Value) -> Result<(), String> {
    let serialized = toml::to_string_pretty(value)
        .map_err(|e| format!("serialize {}: {e}", path.display()))?;
    let mut content = serialized;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    std::fs::write(path, content).map_err(|e| format!("write {}: {e}", path.display()))
}

fn repair_low_coverage_kissconfig_on_disk(work_dir: &Path) -> Result<(), String> {
    let path = work_dir.join(".kissconfig");
    let Some(mut value) = dotfile_backup_state_from_path(&path).and_then(|state| match state {
        DotfileBackupState::Present(payload) => kissconfig_value_needing_threshold_repair(&payload.bytes),
        DotfileBackupState::Missing => None,
    }) else {
        return Ok(());
    };
    let Some(gate) = value.get_mut("gate").and_then(toml::Value::as_table_mut) else {
        return Ok(());
    };
    gate.insert(
        "test_coverage_threshold".to_string(),
        toml::Value::Integer(90),
    );
    write_toml_pretty(&path, &value)
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

fn repair_kissconfig_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut value: toml::Value = text.parse().ok()?;
    if !crate::repo_checks::kissconfig_warn::should_warn_low_test_coverage(&value) {
        return None;
    }
    let gate = value.get_mut("gate").and_then(toml::Value::as_table_mut)?;
    gate.insert(
        "test_coverage_threshold".to_string(),
        toml::Value::Integer(90),
    );
    let serialized = toml::to_string_pretty(&value).ok()?;
    let mut content = serialized;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    Some(content.into_bytes())
}

fn sanitize_malvin_checks_slot(
    work_dir: &Path,
    slot: &mut DotfileBackupState,
) {
    let DotfileBackupState::Present(payload) = slot else {
        return;
    };
    if let Some(fixed) = repair_malvin_checks_bytes(work_dir, &payload.bytes) {
        payload.bytes = fixed;
    }
}

fn sanitize_kissconfig_slot(slot: &mut DotfileBackupState) {
    let DotfileBackupState::Present(payload) = slot else {
        return;
    };
    if let Some(fixed) = repair_kissconfig_bytes(&payload.bytes) {
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
    sanitize_kissconfig_slot(&mut bundle.kissconfig);
    sanitize_malvin_config_slot(&mut bundle.malvin_config);
}

/// Repair known `kiss clamp` damage on disk before gate-loop snapshots.
///
/// Removes bare `kiss` checks (replacing with repo defaults) and raises
/// `gate.test_coverage_threshold` when it is below 90.
pub fn repair_clamp_damaged_dotfiles_on_disk(work_dir: &Path) -> Result<(), String> {
    repair_invalid_malvin_home_config_on_disk(work_dir)?;
    repair_invalid_malvin_checks_on_disk(work_dir)?;
    repair_low_coverage_kissconfig_on_disk(work_dir)
}

#[cfg(test)]
#[path = "gate_restore_repair_tests.rs"]
mod gate_restore_repair_tests;
