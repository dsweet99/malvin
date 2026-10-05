use std::collections::BTreeMap;

use crate::model_id::{ParsedModel, parse_model_id};

use super::MalvinConfig;

const BUILTIN_REMOTES: &[&str] = &[crate::modal_run::MODAL_REMOTE];

type AliasEntries = Vec<Result<(String, String), String>>;

pub(crate) fn parse_model_aliases(text: &str) -> Result<BTreeMap<String, String>, String> {
    let value: toml::Value = text.parse().map_err(|e| format!("invalid TOML: {e}"))?;
    if value.get("nicknames").is_some() {
        return Err("[nicknames] was renamed to [aliases.models]; move its entries there".to_string());
    }
    model_alias_entries(&value)?.into_iter().collect()
}

#[cfg(test)]
pub(crate) fn parse_remote_aliases(text: &str) -> Result<BTreeMap<String, String>, String> {
    let value: toml::Value = text.parse().map_err(|e| format!("invalid TOML: {e}"))?;
    remote_alias_entries(&value)?.into_iter().collect()
}

pub(crate) fn parse_aliases_lenient(text: &str) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let Ok(value) = text.parse::<toml::Value>() else {
        return (BTreeMap::new(), BTreeMap::new());
    };
    (
        keep_valid_aliases(model_alias_entries(&value), "[aliases.models]"),
        keep_valid_aliases(remote_alias_entries(&value), "[aliases.remotes]"),
    )
}

fn keep_valid_aliases(entries: Result<AliasEntries, String>, label: &str) -> BTreeMap<String, String> {
    let warn = |msg: &str| super::warn_config_once(&format!("could not parse {label}: {msg}"));
    let entries = entries.unwrap_or_else(|msg| {
        warn(&msg);
        Vec::new()
    });
    entries
        .into_iter()
        .filter_map(|entry| entry.inspect_err(|msg| warn(&format!("{msg}; skipping it"))).ok())
        .collect()
}

fn model_alias_entries(value: &toml::Value) -> Result<AliasEntries, String> {
    alias_table(value, "models", |name| {
        if name.contains(':') {
            return Err(format!(
                "model alias {name:?} must not contain ':' (aliases are short unprefixed names)"
            ));
        }
        Ok(())
    })
}

fn remote_alias_entries(value: &toml::Value) -> Result<AliasEntries, String> {
    alias_table(value, "remotes", |name| {
        if name.contains([':', '[', ']', ',', '=']) {
            return Err(format!(
                "remote alias {name:?} must not contain ':', '[', ']', ',' or '='"
            ));
        }
        if BUILTIN_REMOTES.contains(&name) {
            return Err(format!("remote alias {name:?} must not shadow a built-in remote"));
        }
        Ok(())
    })
}

fn alias_table(
    root: &toml::Value,
    kind: &str,
    check_name: impl Fn(&str) -> Result<(), String>,
) -> Result<AliasEntries, String> {
    let Some(aliases) = root.get("aliases") else {
        return Ok(Vec::new());
    };
    let Some(table) = aliases.get(kind) else {
        return Ok(Vec::new());
    };
    let table = table
        .as_table()
        .ok_or_else(|| format!("aliases.{kind} must be a table"))?;
    Ok(table
        .iter()
        .map(|(name, target)| alias_entry(kind, name, target, &check_name))
        .collect())
}

fn alias_entry(
    kind: &str,
    name: &str,
    target: &toml::Value,
    check_name: impl Fn(&str) -> Result<(), String>,
) -> Result<(String, String), String> {
    let Some(target) = target.as_str() else {
        return Err(format!("aliases.{kind}.{name} must be a string"));
    };
    let (name, target) = (name.trim(), target.trim());
    if name.is_empty() || target.is_empty() {
        return Err(format!("aliases.{kind} entries must be non-empty"));
    }
    check_name(name)?;
    Ok((name.to_string(), target.to_string()))
}

impl MalvinConfig {
    #[must_use]
    pub fn expand_model_alias<'a>(&'a self, raw: &'a str) -> &'a str {
        let raw = raw.trim();
        self.model_aliases.get(raw).map_or(raw, String::as_str)
    }

    pub fn resolve_model(&self, raw: &str) -> Result<ParsedModel, String> {
        let expanded = self.expand_model_alias(raw);
        parse_model_id(expanded)
    }
}

pub fn parse_model_cli_arg(raw: &str) -> Result<ParsedModel, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    super::load_malvin_config(&cwd).resolve_model(raw)
}

#[must_use]
pub fn load_remote_aliases() -> BTreeMap<String, String> {
    std::env::current_dir()
        .map(|cwd| super::load_malvin_config(&cwd).remote_aliases)
        .unwrap_or_default()
}
