//! Top-level `config.toml` keys outside `[agent]` and `[logs]`.

use crate::terminal_palette::TerminalTheme;

/// Default local llama.cpp context window (`n_ctx` / `n_ctx_seq`).
pub const DEFAULT_CONTEXT_SIZE: u32 = 8192;

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

pub(crate) fn parse_context_size(text: &str) -> Result<u32, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    match super::read_u32(value.get("context_size")) {
        None => Ok(DEFAULT_CONTEXT_SIZE),
        Some(0) => Err("context_size must be positive".to_string()),
        Some(n) => Ok(n),
    }
}

