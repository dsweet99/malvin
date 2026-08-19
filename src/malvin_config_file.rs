use std::collections::BTreeMap;
use std::path::Path;

use crate::log_gc_config::LogsGcConfig;
use crate::output::print_log_warning;
use crate::support_paths::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use crate::terminal_palette::TerminalTheme;
use crate::workspace_paths::malvin_config_path;

#[path = "malvin_config_agent.rs"]
mod malvin_config_agent;
#[path = "malvin_config_default_workflow.rs"]
mod malvin_config_default_workflow;
#[path = "malvin_config_open.rs"]
mod malvin_config_open;
#[path = "malvin_config_parse.rs"]
mod malvin_config_parse;
#[path = "malvin_config_review.rs"]
mod malvin_config_review;
#[path = "malvin_config_top.rs"]
mod malvin_config_top;
pub(crate) use malvin_config_agent::parse_agent_config;
pub(crate) use malvin_config_default_workflow::parse_default_workflow_config;
use malvin_config_open::create_malvin_config_from_template;
pub use malvin_config_open::{
    ensure_malvin_config_file_if_missing, load_agent_config_lenient, load_agent_config_strict,
};
pub(crate) use malvin_config_parse::{
    parse_malvin_config, read_f64, read_string, read_u32, read_u64, read_usize,
};
pub(crate) use malvin_config_review::parse_review_config;
pub use malvin_config_top::{DEFAULT_CONTEXT_SIZE, TokenCostRates};
pub(crate) use malvin_config_top::{parse_context_size, parse_model_token_cost_rates, parse_theme};

pub const DEFAULT_MAX_HYPOTHESES: usize = 5;
pub const DEFAULT_MAX_LOOPS: usize = 1;
pub const DEFAULT_MAX_LOOPS_CODE: usize = 3;
pub const DEFAULT_WRITE_MAX_HYPOTHESES: usize = 10;

const DEFAULT_MALVIN_CONFIG_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/default_malvin_home_config.toml"
));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfig {
    pub model: String,
    pub max_hypotheses: usize,
    pub max_loops: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReviewConfig {
    pub max_hypotheses: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DefaultWorkflowConfig {
    pub max_hypotheses: Option<usize>,
}

impl DefaultWorkflowConfig {
    #[must_use]
    pub fn max_hypotheses_or_default(&self) -> usize {
        self.max_hypotheses.unwrap_or(DEFAULT_MAX_HYPOTHESES)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MalvinConfig {
    pub mem_limit_gb: u64,
    pub context_size: u32,
    pub theme: TerminalTheme,
    pub token_cost_rates: BTreeMap<String, TokenCostRates>,
    pub logs: LogsGcConfig,
    pub agent: AgentConfig,
    pub review: ReviewConfig,
    pub default_workflow: DefaultWorkflowConfig,
}

impl MalvinConfig {
    #[must_use]
    pub fn token_cost_rates_for(&self, model: &str) -> TokenCostRates {
        self.token_cost_rates
            .get(model)
            .copied()
            .unwrap_or_default()
    }
}

pub fn ensure_malvin_config_file(work_dir: &Path) -> Result<(), String> {
    let _ = open_malvin_config(work_dir)?;
    Ok(())
}

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

pub fn open_malvin_config(work_dir: &Path) -> Result<MalvinConfig, String> {
    let path = malvin_config_path(work_dir);
    ensure_config_parent_dir(&path)?;
    let template = parse_template_value()?;
    if path.is_file() {
        let meta = std::fs::metadata(&path).map_err(|e| format!("stat {}: {e}", path.display()))?;
        if meta.len() == 0 {
            std::fs::remove_file(&path)
                .map_err(|e| format!("remove empty {}: {e}", path.display()))?;
        }
    }
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
    if path == crate::workspace_paths::malvin_home_config_path() {
        crate::workspace_paths::assert_home_malvin_config_disk_io_allowed("create")?;
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
    if path == crate::workspace_paths::malvin_home_config_path() {
        crate::workspace_paths::assert_home_malvin_config_disk_io_allowed("write")?;
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

#[cfg(test)]
#[path = "malvin_config_file_tests_model_prefix.rs"]
mod malvin_config_file_tests_model_prefix;

#[cfg(test)]
#[path = "malvin_config_file_tests.rs"]
mod malvin_config_file_tests;

#[cfg(test)]
#[path = "malvin_config_file_tests_parse.rs"]
mod malvin_config_file_tests_parse;

#[cfg(test)]
#[path = "malvin_config_file_tests_no_overwrite.rs"]
mod malvin_config_file_tests_no_overwrite;
