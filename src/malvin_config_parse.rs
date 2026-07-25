//! Parse typed `MalvinConfig` from TOML text and shared value readers.

use crate::log_gc_config::{parse_logs_gc_config, LogsGcConfig};
use crate::mem_limit_config::{default_mem_limit_gb, parse_mem_limit_gb};
use crate::output::print_log_warning;
use crate::terminal_palette::TerminalTheme;

use super::{
    parse_agent_config, parse_context_size, parse_review_config, parse_theme, AgentConfig,
    MalvinConfig, ReviewConfig, DEFAULT_CONTEXT_SIZE,
};

pub(crate) fn parse_malvin_config(text: &str) -> MalvinConfig {
    let mem_limit_gb = parse_or_warn(parse_mem_limit_gb(text), "mem_limit_gb", default_mem_limit_gb());
    let context_size =
        parse_or_warn(parse_context_size(text), "context_size", DEFAULT_CONTEXT_SIZE);
    let logs = parse_or_warn(parse_logs_gc_config(text), "[logs]", LogsGcConfig::default());
    let agent = parse_or_warn(parse_agent_config(text), "[agent]", AgentConfig::default());
    let review = parse_or_warn(parse_review_config(text), "[review]", ReviewConfig::default());
    let theme = parse_or_warn(parse_theme(text), "theme", TerminalTheme::Dark);
    MalvinConfig {
        mem_limit_gb,
        context_size,
        theme,
        logs,
        agent,
        review,
    }
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
