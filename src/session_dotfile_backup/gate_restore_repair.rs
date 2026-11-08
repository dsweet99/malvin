use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::output::print_log_warning;

fn malvin_home_config_bytes_need_repair(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return true;
    };
    text.parse::<toml::Value>().is_err()
}

fn invalid_config_backup_path(config: &Path) -> PathBuf {
    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let id = super::alloc::random_backup_id(0);
    let mut name = config.as_os_str().to_os_string();
    name.push(format!(".invalid.{stamp}_{id}"));
    PathBuf::from(name)
}

fn set_aside_invalid_config(config: &Path) -> Result<(), String> {
    let backup = invalid_config_backup_path(config);
    match std::fs::rename(config, &backup) {
        Ok(()) => {
            print_log_warning(&format!(
                "{} is not valid TOML, so malvin moved it to {} and wrote a default config in its place; fix the saved file and move it back to restore your settings",
                config.display(),
                backup.display()
            ));
            Ok(())
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "move {} to {}: {e}",
            config.display(),
            backup.display()
        )),
    }
}

fn repair_malvin_home_config_on_disk_impl(work_dir: &Path) -> Result<(), String> {
    if !crate::workspace::workspace_paths::home_malvin_config_delete_allowed() {
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
    if bytes.is_empty() {
        super::alloc::remove_if_exists(&path, "malvin home config repair")
            .map_err(|e| format!("remove {}: {e}", path.display()))?;
    } else {
        set_aside_invalid_config(&path)?;
    }
    crate::config::malvin_config_file::ensure_malvin_config_file_if_missing(work_dir)
}

pub fn repair_invalid_malvin_home_config_on_disk(work_dir: &Path) -> Result<(), String> {
    repair_malvin_home_config_on_disk_impl(work_dir)
}

#[cfg(test)]
#[path = "gate_restore_repair_tests.rs"]
mod gate_restore_repair_tests;
