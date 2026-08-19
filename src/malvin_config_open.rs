use std::path::Path;

use super::{
    AgentConfig, MalvinConfig, ensure_config_parent_dir, merge_missing_keys, parse_agent_config,
    parse_malvin_config, parse_template_value, write_config_value,
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

pub fn ensure_malvin_config_file_if_missing(work_dir: &Path) -> Result<(), String> {
    let path = malvin_config_path(work_dir);
    ensure_config_parent_dir(&path)?;
    if path.is_file() && !file_is_empty(&path)? {
        return Ok(());
    }
    if path.is_file() {
        std::fs::remove_file(&path).map_err(|e| format!("remove empty {}: {e}", path.display()))?;
    }
    let template = parse_template_value()?;
    let _ = create_malvin_config_from_template(&path, &template)?;
    Ok(())
}

fn file_is_empty(path: &Path) -> Result<bool, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("stat {}: {e}", path.display()))?;
    Ok(meta.len() == 0)
}

pub fn load_agent_config_strict(work_dir: &Path) -> Result<AgentConfig, String> {
    let path = malvin_config_path(work_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(AgentConfig::default());
    };
    let Ok(template) = parse_template_value() else {
        return parse_agent_config(&text);
    };
    let Ok(mut on_disk) = text.parse::<toml::Value>() else {
        return Err(format!("invalid TOML in {}", path.display()));
    };
    let _ = merge_missing_keys(&mut on_disk, &template);
    let merged = toml::to_string(&on_disk).map_err(|e| e.to_string())?;
    parse_agent_config(&merged)
}

#[must_use]
pub fn load_agent_config_lenient(work_dir: &Path) -> AgentConfig {
    if let Ok(agent) = load_agent_config_strict(work_dir) {
        return agent;
    }
    let path = malvin_config_path(work_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return AgentConfig::default();
    };
    let Ok(template) = parse_template_value() else {
        return AgentConfig::default();
    };
    let Ok(mut on_disk) = text.parse::<toml::Value>() else {
        return AgentConfig::default();
    };
    let _ = merge_missing_keys(&mut on_disk, &template);
    if let Some(table) = on_disk.get_mut("agent").and_then(toml::Value::as_table_mut) {
        table.insert(
            "model".into(),
            toml::Value::String(AgentConfig::default().model),
        );
    }
    let Ok(merged) = toml::to_string(&on_disk) else {
        return AgentConfig::default();
    };
    parse_agent_config(&merged).unwrap_or_default()
}
