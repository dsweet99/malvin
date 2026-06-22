//! Unified `~/.malvin_home/config.toml` schema, default merge-on-open, and typed accessors.

use std::path::Path;

use crate::log_gc_config::{LogsGcConfig, parse_logs_gc_config};
use crate::terminal_palette::TerminalTheme;
use crate::mem_limit_config::{default_mem_limit_gb, parse_mem_limit_gb};
use crate::output::print_log_warning;
use crate::support_paths::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use crate::workspace_paths::malvin_config_path;

#[path = "malvin_config_open.rs"]
mod malvin_config_open;
pub use malvin_config_open::ensure_malvin_config_file_if_missing;
use malvin_config_open::create_malvin_config_from_template;

pub const DEFAULT_MAX_HYPOTHESES: usize = 5;
pub const DEFAULT_MAX_LOOPS: usize = 1;
pub const DEFAULT_MAX_LOOPS_CODE: usize = 3;

const DEFAULT_MALVIN_CONFIG_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/config.toml"
));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfig {
    pub model: String,
    pub max_hypotheses: usize,
    /// Gate-loop budget for kpop and bare invocation.
    pub max_loops: usize,
    /// Gate-loop budget for code and tidy.
    pub max_loops_code: usize,
    pub max_acp_retries: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: DEFAULT_CLI_MODEL.to_string(),
            max_hypotheses: DEFAULT_MAX_HYPOTHESES,
            max_loops: DEFAULT_MAX_LOOPS,
            max_loops_code: DEFAULT_MAX_LOOPS_CODE,
            max_acp_retries: DEFAULT_MAX_ACP_RETRIES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalvinConfig {
    pub mem_limit_gb: u64,
    pub theme: TerminalTheme,
    pub logs: LogsGcConfig,
    pub agent: AgentConfig,
}

/// Ensure `~/.malvin_home/config.toml` exists and contains every known key (writes missing defaults).
pub fn ensure_malvin_config_file(work_dir: &Path) -> Result<(), String> {
    let _ = open_malvin_config(work_dir)?;
    Ok(())
}

/// Read workspace config without creating or updating the on-disk file.
pub fn load_malvin_config(work_dir: &Path) -> MalvinConfig {
    let path = malvin_config_path(work_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return parse_malvin_config(DEFAULT_MALVIN_CONFIG_TEMPLATE);
    };
    let Ok(template) = parse_template_value() else {
        return parse_malvin_config(&text);
    };
    let Ok(mut on_disk) = text.parse::<toml::Value>() else {
        print_log_warning(&format!("invalid TOML in {}", path.display()));
        return parse_malvin_config(DEFAULT_MALVIN_CONFIG_TEMPLATE);
    };
    let _ = merge_missing_keys(&mut on_disk, &template);
    let merged = toml::to_string(&on_disk).unwrap_or(text);
    parse_malvin_config(&merged)
}

/// Open workspace config: create if missing (with template defaults), never rewrite an existing file.
pub fn open_malvin_config(work_dir: &Path) -> Result<MalvinConfig, String> {
    let path = malvin_config_path(work_dir);
    ensure_config_parent_dir(&path)?;
    let template = parse_template_value()?;
    if !path.is_file() {
        return create_malvin_config_from_template(&path, &template);
    }
    let mut on_disk = read_on_disk_config_value(&path)?;
    merge_missing_keys(&mut on_disk, &template);
    Ok(parse_malvin_config(
        &toml::to_string(&on_disk).map_err(|e| e.to_string())?,
    ))
}

pub(crate) fn ensure_config_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    Ok(())
}

pub(crate) fn read_on_disk_config_value(path: &Path) -> Result<toml::Value, String> {
    if !path.is_file() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    text.parse::<toml::Value>()
        .map_err(|e| format!("invalid TOML in {}: {e}", path.display()))
}

pub(crate) fn write_config_value(path: &Path, value: &toml::Value) -> Result<(), String> {
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
                if let Some(existing) = into_table.get_mut(key) {
                    if merge_missing_keys(existing, template_value) {
                        changed = true;
                    }
                }
            }
            changed
        }
        _ => false,
    }
}

pub(crate) fn parse_malvin_config(text: &str) -> MalvinConfig {
    let mem_limit_gb = parse_mem_limit_gb(text).unwrap_or_else(|msg| {
        print_log_warning(&format!("could not parse mem_limit_gb: {msg}"));
        default_mem_limit_gb()
    });
    let logs = parse_logs_gc_config(text).unwrap_or_else(|msg| {
        print_log_warning(&format!("could not parse [logs]: {msg}"));
        LogsGcConfig::default()
    });
    let agent = parse_agent_config(text).unwrap_or_else(|msg| {
        print_log_warning(&format!("could not parse [agent]: {msg}"));
        AgentConfig::default()
    });
    let theme = parse_theme(text).unwrap_or_else(|msg| {
        print_log_warning(&format!("could not parse theme: {msg}"));
        TerminalTheme::Dark
    });
    MalvinConfig {
        mem_limit_gb,
        theme,
        logs,
        agent,
    }
}

pub(crate) fn parse_theme(text: &str) -> Result<TerminalTheme, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let Some(raw) = read_string(value.get("theme")) else {
        return Ok(TerminalTheme::Dark);
    };
    match raw.to_ascii_lowercase().as_str() {
        "dark" => Ok(TerminalTheme::Dark),
        "light" => Ok(TerminalTheme::Light),
        other => Err(format!("unsupported theme {other:?}; use \"dark\" or \"light\"")),
    }
}

pub(crate) fn parse_agent_config(text: &str) -> Result<AgentConfig, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let agent = value
        .get("agent")
        .ok_or_else(|| "missing [agent] section".to_string())?;
    Ok(agent_config_from_table(agent))
}

pub(crate) fn agent_config_from_table(agent: &toml::Value) -> AgentConfig {
    let defaults = AgentConfig::default();
    AgentConfig {
        model: read_string(agent.get("model")).unwrap_or(defaults.model),
        max_hypotheses: read_usize(agent.get("max_hypotheses")).unwrap_or(defaults.max_hypotheses),
        max_loops: read_usize(agent.get("max_loops")).unwrap_or(defaults.max_loops),
        max_loops_code: read_usize(agent.get("max_loops_code")).unwrap_or(defaults.max_loops_code),
        max_acp_retries: read_u32(agent.get("max_acp_retries")).unwrap_or(defaults.max_acp_retries),
    }
}

pub(crate) fn read_string(value: Option<&toml::Value>) -> Option<String> {
    value?.as_str().map(str::to_string)
}

fn parse_toml_integer(value: Option<&toml::Value>) -> Option<i64> {
    let v = value?;
    if let Some(i) = v.as_integer() {
        return Some(i);
    }
    v.as_str()?.parse().ok()
}

pub(crate) fn read_usize(value: Option<&toml::Value>) -> Option<usize> {
    parse_toml_integer(value).and_then(|i| usize::try_from(i).ok())
}

pub(crate) fn read_u32(value: Option<&toml::Value>) -> Option<u32> {
    parse_toml_integer(value).and_then(|i| u32::try_from(i).ok())
}

pub(crate) fn read_u64(value: Option<&toml::Value>) -> Option<u64> {
    parse_toml_integer(value).and_then(|i| u64::try_from(i).ok())
}
