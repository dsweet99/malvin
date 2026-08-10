//! Parse typed `MalvinConfig` from TOML text and shared value readers.

use crate::log_gc_config::{parse_logs_gc_config, LogsGcConfig};
use crate::mem_limit_config::{default_mem_limit_gb, parse_mem_limit_gb};
use crate::output::print_log_warning;
use crate::terminal_palette::TerminalTheme;

use std::collections::BTreeMap;

use super::{
    parse_agent_config, parse_context_size, parse_default_workflow_config,
    parse_model_token_cost_rates, parse_review_config, parse_theme, AgentConfig,
    DefaultWorkflowConfig, MalvinConfig, ReviewConfig, DEFAULT_CONTEXT_SIZE,
};

pub(crate) fn parse_malvin_config(text: &str) -> MalvinConfig {
    let (mem_limit_gb, context_size, theme) = parse_top_level_keys(text);
    let token_cost_rates = parse_or_warn(
        parse_model_token_cost_rates(text),
        "[agent.*.*] usd_per_microtoken_*",
        BTreeMap::new(),
    );
    let (logs, agent, review, default_workflow) = parse_config_sections(text);
    MalvinConfig {
        mem_limit_gb,
        context_size,
        theme,
        token_cost_rates,
        logs,
        agent,
        review,
        default_workflow,
    }
}

fn parse_top_level_keys(text: &str) -> (u64, u32, TerminalTheme) {
    (
        parse_or_warn(parse_mem_limit_gb(text), "mem_limit_gb", default_mem_limit_gb()),
        parse_or_warn(parse_context_size(text), "context_size", DEFAULT_CONTEXT_SIZE),
        parse_or_warn(parse_theme(text), "theme", TerminalTheme::Dark),
    )
}

fn parse_config_sections(
    text: &str,
) -> (
    LogsGcConfig,
    AgentConfig,
    ReviewConfig,
    DefaultWorkflowConfig,
) {
    (
        parse_or_warn(parse_logs_gc_config(text), "[logs]", LogsGcConfig::default()),
        parse_or_warn(parse_agent_config(text), "[agent]", AgentConfig::default()),
        parse_or_warn(parse_review_config(text), "[review]", ReviewConfig::default()),
        parse_or_warn(
            parse_default_workflow_config(text),
            "[default_workflow]",
            DefaultWorkflowConfig::default(),
        ),
    )
}

fn parse_or_warn<T>(result: Result<T, String>, label: &str, fallback: T) -> T {
    result.unwrap_or_else(|msg| {
        print_log_warning(&format!("could not parse {label}: {msg}"));
        fallback
    })
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

pub(crate) fn read_f64(value: Option<&toml::Value>) -> Option<f64> {
    let v = value?;
    if let Some(f) = v.as_float() {
        return Some(f);
    }
    if let Some(i) = v.as_integer() {
        #[allow(clippy::cast_precision_loss)]
        return Some(i as f64);
    }
    v.as_str()?.parse().ok()
}
