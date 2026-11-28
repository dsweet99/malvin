use crate::output::print_log_warning;
use crate::workspace::workspace_paths::malvin_home_config_path;

#[path = "malvin_config_agent.rs"]
mod malvin_config_agent;
#[path = "malvin_config_model_policy.rs"]
mod malvin_config_model_policy;
#[path = "malvin_config_open.rs"]
mod malvin_config_open;
#[path = "malvin_config_parse.rs"]
mod malvin_config_parse;
#[path = "malvin_config_top.rs"]
mod malvin_config_top;
pub(crate) use super::malvin_config_io::{
    DEFAULT_MALVIN_CONFIG_TEMPLATE, ensure_config_parent_dir, merge_missing_keys,
    parse_template_value, read_on_disk_config_value, write_config_value,
};
pub(crate) use super::malvin_config_loaders::load_model_aliases;
pub use super::malvin_config_loaders::{
    load_logs_gc_config, load_mem_limit_bytes, load_mem_limit_gb, load_remote_aliases,
    parse_model_cli_arg, parse_model_list_cli_arg,
};
pub use super::malvin_config_types::{AgentConfig, MalvinConfig};
pub(crate) use super::malvin_config_values::{
    read_f64, read_string, read_u32, read_u64, warn_config_once,
};
pub(crate) use malvin_config_agent::parse_agent_config;
#[cfg(test)]
pub(crate) use malvin_config_model_policy::parse_remote_aliases;
pub(crate) use malvin_config_model_policy::{parse_aliases_lenient, parse_model_aliases};
use malvin_config_open::create_malvin_config_from_template;
pub use malvin_config_open::{
    ensure_malvin_config_file_if_missing, load_agent_config_lenient, load_agent_config_strict,
};
pub(crate) use malvin_config_parse::parse_malvin_config;
pub use malvin_config_top::{
    DEFAULT_CONTEXT_SIZE, DEFAULT_TIME_SINCE_LAST_CALL_SECONDS, TokenCostRates,
};
pub(crate) use malvin_config_top::{
    parse_context_size, parse_model_token_cost_rates, parse_theme,
    parse_time_since_last_call_seconds,
};

pub const DEFAULT_MAX_LOOPS: usize = 999;

pub fn ensure_malvin_config_file() -> Result<(), String> {
    let _ = open_malvin_config()?;
    Ok(())
}

pub fn load_malvin_config() -> MalvinConfig {
    let path = malvin_home_config_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        let mut cfg = parse_malvin_config(DEFAULT_MALVIN_CONFIG_TEMPLATE);
        cfg.mem_limit_gb = crate::config::mem_limit_config::default_mem_limit_gb();
        return cfg;
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

pub fn open_malvin_config() -> Result<MalvinConfig, String> {
    let path = malvin_home_config_path();
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
#[path = "malvin_config_file_tests_aliases.rs"]
mod malvin_config_file_tests_aliases;

#[cfg(test)]
#[path = "malvin_config_file_tests_no_overwrite.rs"]
mod malvin_config_file_tests_no_overwrite;
