use std::path::Path;

pub(crate) const DEFAULT_MALVIN_CONFIG_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/default_malvin_home_config.toml"
));

pub(crate) fn ensure_config_parent_dir(path: &Path) -> Result<(), String> {
    if path == crate::workspace::workspace_paths::malvin_home_config_path() {
        crate::workspace::workspace_paths::assert_home_malvin_config_disk_io_allowed("create")?;
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    Ok(())
}

pub(crate) fn read_on_disk_config_value(path: &Path) -> Result<toml::Value, String> {
    if !path.is_file() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    text.parse::<toml::Value>()
        .map_err(|e| format!("invalid TOML in {}: {e}", path.display()))
}

pub(crate) fn write_config_value(path: &Path, value: &toml::Value) -> Result<(), String> {
    if path == crate::workspace::workspace_paths::malvin_home_config_path() {
        crate::workspace::workspace_paths::assert_home_malvin_config_disk_io_allowed("write")?;
    }
    let serialized =
        toml::to_string_pretty(value).map_err(|e| format!("serialize {}: {e}", path.display()))?;
    let mut content = serialized;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    std::fs::write(path, &content).map_err(|e| format!("write {}: {e}", path.display()))
}

pub(crate) fn parse_template_value() -> Result<toml::Value, String> {
    DEFAULT_MALVIN_CONFIG_TEMPLATE
        .parse()
        .map_err(|e| format!("invalid bundled config template: {e}"))
}

pub(crate) fn merge_missing_keys(into: &mut toml::Value, template: &toml::Value) -> bool {
    match (into, template) {
        (toml::Value::Table(into_table), toml::Value::Table(template_table)) => {
            let mut changed = false;
            for (key, template_value) in template_table {
                if !into_table.contains_key(key) {
                    into_table.insert(key.clone(), template_value.clone());
                    changed = true;
                    continue;
                }
                if let Some(existing) = into_table.get_mut(key)
                    && merge_missing_keys(existing, template_value)
                {
                    changed = true;
                }
            }
            changed
        }
        _ => false,
    }
}
