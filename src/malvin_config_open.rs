//! Helpers for [`super::open_malvin_config`] (keeps `malvin_config_file.rs` under kiss line limits).

use std::path::Path;

use super::{
    ensure_config_parent_dir, merge_missing_keys, parse_malvin_config, parse_template_value,
    write_config_value, MalvinConfig,
};
use crate::workspace_paths::malvin_config_path;

pub(super) fn create_malvin_config_from_template(
    path: &Path,
    template: &toml::Value,
) -> Result<MalvinConfig, String> {
    let mut on_disk = toml::Value::Table(toml::map::Map::new());
    merge_missing_keys(&mut on_disk, template);
    write_config_value(path, &on_disk)?;
    Ok(parse_malvin_config(
        &toml::to_string(&on_disk).map_err(|e| e.to_string())?,
    ))
}

/// Create workspace config only when the file is absent; never read or rewrite an existing file.
///
/// Used when snapshotting dotfiles after an agent session so tampered invalid TOML can still be
/// backed up and restored without failing the gate loop.
pub fn ensure_malvin_config_file_if_missing(work_dir: &Path) -> Result<(), String> {
    let path = malvin_config_path(work_dir);
    ensure_config_parent_dir(&path)?;
    if path.is_file() {
        return Ok(());
    }
    let template = parse_template_value()?;
    let _ = create_malvin_config_from_template(&path, &template)?;
    Ok(())
}
