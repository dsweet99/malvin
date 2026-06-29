//! Top-level `config.toml` keys outside `[agent]` and `[logs]`.

use crate::terminal_palette::TerminalTheme;

pub(crate) fn parse_theme(text: &str) -> Result<TerminalTheme, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let Some(raw) = super::read_string(value.get("theme")) else {
        return Ok(TerminalTheme::Dark);
    };
    match raw.to_ascii_lowercase().as_str() {
        "dark" => Ok(TerminalTheme::Dark),
        "light" => Ok(TerminalTheme::Light),
        other => Err(format!("unsupported theme {other:?}; use \"dark\" or \"light\"")),
    }
}

pub(crate) fn read_bool(value: Option<&toml::Value>) -> Option<bool> {
    let v = value?;
    if let Some(b) = v.as_bool() {
        return Some(b);
    }
    match v.as_str()?.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

pub(crate) fn parse_mpc(text: &str) -> Result<bool, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    Ok(read_bool(value.get("mpc")).unwrap_or(true))
}
