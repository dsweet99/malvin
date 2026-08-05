//! Top-level `config.toml` keys outside `[agent]` and `[logs]`.

use std::collections::BTreeMap;

use crate::terminal_palette::TerminalTheme;

/// Default local llama.cpp context window (`n_ctx` / `n_ctx_seq`).
pub const DEFAULT_CONTEXT_SIZE: u32 = 8192;

/// Tokens per microtoken (USD rates are dollars per million tokens).
pub const TOKENS_PER_MICROTOKEN: f64 = 1_000_000.0;

/// Per-microtoken USD rates for estimating Cursor-mode run cost.
/// Field names match `~/.malvin_home/config.toml` keys under `[agent.<provider>.<name>]`.
/// One microtoken = one million tokens (industry dollars-per-MTok units).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(clippy::struct_field_names)]
pub struct TokenCostRates {
    pub usd_per_microtoken_in: f64,
    pub usd_per_microtoken_out: f64,
    pub usd_per_microtoken_cache_read: f64,
    pub usd_per_microtoken_cache_write: f64,
}

impl TokenCostRates {
    /// USD components: `(cost_in, cost_out, cost_read, cost_write)`.
    ///
    /// `input_tokens` is non-cache prompt/input only (cache billed via read/write rates).
    /// Each component is `usd_per_microtoken_* × tokens / 1_000_000`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn estimate_components(
        self,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> (f64, f64, f64, f64) {
        (
            (input_tokens as f64) * self.usd_per_microtoken_in / TOKENS_PER_MICROTOKEN,
            (output_tokens as f64) * self.usd_per_microtoken_out / TOKENS_PER_MICROTOKEN,
            (cache_read_tokens as f64) * self.usd_per_microtoken_cache_read / TOKENS_PER_MICROTOKEN,
            (cache_write_tokens as f64) * self.usd_per_microtoken_cache_write / TOKENS_PER_MICROTOKEN,
        )
    }

    #[must_use]
    pub fn estimate_usd(
        self,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> f64 {
        let (cost_in, cost_out, cost_read, cost_write) = self.estimate_components(
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_write_tokens,
        );
        cost_in + cost_out + cost_read + cost_write
    }
}

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

/// Parse `[agent.<provider>.<name>]` tables that define `usd_per_microtoken_*` rates.
///
/// Keys in the map are full model ids (`cursor:auto`, `openrouter:org/model`, …).
pub(crate) fn parse_model_token_cost_rates(
    text: &str,
) -> Result<BTreeMap<String, TokenCostRates>, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let Some(agent) = value.get("agent").and_then(toml::Value::as_table) else {
        return Ok(BTreeMap::new());
    };
    let mut out = BTreeMap::new();
    for (provider, provider_val) in agent {
        let Some(models) = provider_val.as_table() else {
            continue;
        };
        for (model_name, model_val) in models {
            let Some(table) = model_val.as_table() else {
                continue;
            };
            if !table.keys().any(|k| k.starts_with("usd_per_microtoken")) {
                continue;
            }
            let rates = token_cost_rates_from_value(model_val)?;
            out.insert(format!("{provider}:{model_name}"), rates);
        }
    }
    Ok(out)
}

pub(crate) fn token_cost_rates_from_value(value: &toml::Value) -> Result<TokenCostRates, String> {
    Ok(TokenCostRates {
        usd_per_microtoken_in: non_negative_rate(
            value.get("usd_per_microtoken_in"),
            "usd_per_microtoken_in",
        )?,
        usd_per_microtoken_out: non_negative_rate(
            value.get("usd_per_microtoken_out"),
            "usd_per_microtoken_out",
        )?,
        usd_per_microtoken_cache_read: non_negative_rate(
            value.get("usd_per_microtoken_cache_read"),
            "usd_per_microtoken_cache_read",
        )?,
        usd_per_microtoken_cache_write: non_negative_rate(
            value.get("usd_per_microtoken_cache_write"),
            "usd_per_microtoken_cache_write",
        )?,
    })
}

fn non_negative_rate(value: Option<&toml::Value>, key: &str) -> Result<f64, String> {
    let Some(v) = value else {
        return Ok(0.0);
    };
    let Some(n) = super::read_f64(Some(v)) else {
        return Err(format!("{key} must be a number"));
    };
    if n < 0.0 {
        return Err(format!("{key} must be >= 0"));
    }
    Ok(n)
}
