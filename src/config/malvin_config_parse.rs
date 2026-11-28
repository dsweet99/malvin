use crate::config::log_gc_config::{LogsGcConfig, parse_logs_gc_config};
use crate::config::mem_limit_config::{default_mem_limit_gb, parse_mem_limit_gb};
use crate::terminal_palette::TerminalTheme;

use std::collections::BTreeMap;

use super::{
    AgentConfig, DEFAULT_CONTEXT_SIZE, DEFAULT_TIME_SINCE_LAST_CALL_SECONDS, MalvinConfig,
    parse_agent_config, parse_aliases_lenient, parse_context_size, parse_model_token_cost_rates,
    parse_theme, parse_time_since_last_call_seconds, warn_config_once,
};

pub(crate) fn parse_malvin_config(text: &str) -> MalvinConfig {
    let (mem_limit_gb, context_size, theme, auto_setup_costs) = parse_top_level_keys(text);
    let (model_aliases, remote_aliases) = parse_aliases_lenient(text);
    let token_cost_rates = parse_token_cost_rates(text);
    let (logs, agent) = parse_config_sections(text, &model_aliases);
    MalvinConfig {
        mem_limit_gb,
        context_size,
        theme,
        auto_setup_costs,
        model_aliases,
        remote_aliases,
        token_cost_rates,
        logs,
        agent,
        dm_on_emit: parse_dm_on_emit(text),
        time_since_last_call_seconds: parse_or_warn(
            parse_time_since_last_call_seconds(text),
            "time_since_last_call_seconds",
            DEFAULT_TIME_SINCE_LAST_CALL_SECONDS,
        ),
    }
}

fn parse_token_cost_rates(text: &str) -> BTreeMap<String, super::TokenCostRates> {
    let Some(parsed) = parse_or_warn(
        parse_model_token_cost_rates(text).map(Some),
        "[agent.*.*] usd_per_megatoken_*",
        None,
    ) else {
        return BTreeMap::new();
    };
    for error in &parsed.table_errors {
        warn_config_once(&format!("could not parse token costs: {error}"));
    }
    parsed.rates
}

fn parse_dm_on_emit(text: &str) -> Option<String> {
    let value = text.parse::<toml::Value>().ok()?;
    let raw = value.get("dm").and_then(|section| section.get("on_emit"))?;
    if raw.as_array().is_some_and(Vec::is_empty) {
        return None;
    }
    match raw.as_str() {
        Some(command) if !command.trim().is_empty() => Some(command.to_string()),
        Some(_) => None,
        None => {
            warn_config_once("[dm].on_emit must be a string");
            None
        }
    }
}

fn parse_top_level_keys(text: &str) -> (u64, u32, TerminalTheme, bool) {
    (
        parse_or_warn(
            parse_mem_limit_gb(text),
            "mem_limit_gb",
            default_mem_limit_gb(),
        ),
        parse_or_warn(
            parse_context_size(text),
            "context_size",
            DEFAULT_CONTEXT_SIZE,
        ),
        parse_or_warn(parse_theme(text), "theme", TerminalTheme::Dark),
        parse_auto_setup_costs(text),
    )
}

fn parse_auto_setup_costs(text: &str) -> bool {
    let Ok(value) = text.parse::<toml::Value>() else {
        return true;
    };
    value.get("auto_setup_costs").is_none_or(|value| {
        value.as_bool().unwrap_or_else(|| {
            warn_config_once("could not parse auto_setup_costs: must be a boolean");
            true
        })
    })
}

fn parse_config_sections(
    text: &str,
    model_aliases: &BTreeMap<String, String>,
) -> (LogsGcConfig, AgentConfig) {
    (
        parse_or_warn(
            parse_logs_gc_config(text),
            "[logs]",
            LogsGcConfig::default(),
        ),
        parse_or_warn(
            parse_agent_config(text, model_aliases),
            "[agent]",
            AgentConfig::default(),
        ),
    )
}

fn parse_or_warn<T>(result: Result<T, String>, label: &str, fallback: T) -> T {
    result.unwrap_or_else(|msg| {
        warn_config_once(&format!("could not parse {label}: {msg}"));
        fallback
    })
}
