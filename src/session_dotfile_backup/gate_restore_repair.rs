
use std::path::Path;

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

pub fn repair_invalid_malvin_home_config_on_disk(work_dir: &Path) -> Result<(), String> {
    repair_malvin_home_config_on_disk_impl(work_dir)
}

#[cfg(test)]
#[path = "gate_restore_repair_tests.rs"]
mod gate_restore_repair_tests;
