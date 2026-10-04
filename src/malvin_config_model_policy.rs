use std::collections::BTreeMap;

use crate::model_id::{ParsedModel, parse_model_id};

use super::MalvinConfig;

pub(crate) fn parse_nicknames(text: &str) -> Result<BTreeMap<String, String>, String> {
    let value: toml::Value = text.parse().map_err(|e| format!("invalid TOML: {e}"))?;
    let Some(table) = value.get("nicknames").and_then(toml::Value::as_table) else {
        return Ok(BTreeMap::new());
    };
    let mut out = BTreeMap::new();
    for (nick, target) in table {
        let (nick, target) = nickname_entry(nick, target)?;
        out.insert(nick, target);
    }
    Ok(out)
}

fn nickname_entry(nick: &str, target: &toml::Value) -> Result<(String, String), String> {
    let Some(target) = target.as_str() else {
        return Err(format!("nicknames.{nick} must be a string"));
    };
    let nick = nick.trim();
    let target = target.trim();
    if nick.is_empty() || target.is_empty() {
        return Err("nicknames entries must be non-empty".to_string());
    }
    if nick.contains(':') {
        return Err(format!(
            "nickname {nick:?} must not contain ':' (nicknames are short unprefixed names)"
        ));
    }
    Ok((nick.to_string(), target.to_string()))
}

impl MalvinConfig {
    #[must_use]
    pub fn expand_nickname<'a>(&'a self, raw: &'a str) -> &'a str {
        let raw = raw.trim();
        self.nicknames.get(raw).map_or(raw, String::as_str)
    }

    pub fn resolve_model(&self, raw: &str) -> Result<ParsedModel, String> {
        let expanded = self.expand_nickname(raw);
        parse_model_id(expanded)
    }
}

pub fn parse_model_cli_arg(raw: &str) -> Result<ParsedModel, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    super::load_malvin_config(&cwd).resolve_model(raw)
}
