use std::collections::BTreeMap;

use crate::terminal_palette::TerminalTheme;

pub const DEFAULT_CONTEXT_SIZE: u32 = 8192;

pub const TOKENS_PER_MEGATOKEN: f64 = 1_000_000.0;
const LEGACY_RATE_PREFIX: &str = "usd_per_microtoken";

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(clippy::struct_field_names)]
pub struct TokenCostRates {
    pub usd_per_megatoken_in: f64,
    pub usd_per_megatoken_out: f64,
    pub usd_per_megatoken_cache_read: f64,
    pub usd_per_megatoken_cache_write: f64,
}

impl TokenCostRates {
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
            (input_tokens as f64) * self.usd_per_megatoken_in / TOKENS_PER_MEGATOKEN,
            (output_tokens as f64) * self.usd_per_megatoken_out / TOKENS_PER_MEGATOKEN,
            (cache_read_tokens as f64) * self.usd_per_megatoken_cache_read / TOKENS_PER_MEGATOKEN,
            (cache_write_tokens as f64) * self.usd_per_megatoken_cache_write / TOKENS_PER_MEGATOKEN,
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
    let value: toml::Value = text.parse().map_err(|e| format!("invalid TOML: {e}"))?;
    let Some(raw) = super::read_string(value.get("theme")) else {
        return Ok(TerminalTheme::Dark);
    };
    match raw.to_ascii_lowercase().as_str() {
        "dark" => Ok(TerminalTheme::Dark),
        "light" => Ok(TerminalTheme::Light),
        other => Err(format!(
            "unsupported theme {other:?}; use \"dark\" or \"light\""
        )),
    }
}

pub(crate) fn parse_context_size(text: &str) -> Result<u32, String> {
    let value: toml::Value = text.parse().map_err(|e| format!("invalid TOML: {e}"))?;
    match super::read_u32(value.get("context_size")) {
        None => Ok(DEFAULT_CONTEXT_SIZE),
        Some(0) => Err("context_size must be positive".to_string()),
        Some(n) => Ok(n),
    }
}

pub(crate) struct ModelTokenCostRates {
    pub rates: BTreeMap<String, TokenCostRates>,
    pub table_errors: Vec<String>,
}

pub(crate) fn parse_model_token_cost_rates(text: &str) -> Result<ModelTokenCostRates, String> {
    let value: toml::Value = text.parse().map_err(|e| format!("invalid TOML: {e}"))?;
    let mut out = ModelTokenCostRates {
        rates: BTreeMap::new(),
        table_errors: Vec::new(),
    };
    let Some(agent) = value.get("agent").and_then(toml::Value::as_table) else {
        return Ok(out);
    };
    for (provider, provider_val) in agent {
        let Some(models) = provider_val.as_table() else {
            continue;
        };
        for (model_name, model_val) in models {
            if let Some(table) = model_val.as_table() {
                add_model_rates(&mut out, (provider, model_name), table, model_val);
            }
        }
    }
    Ok(out)
}

fn add_model_rates(
    out: &mut ModelTokenCostRates,
    (provider, model_name): (&str, &str),
    table: &toml::map::Map<String, toml::Value>,
    model_val: &toml::Value,
) {
    let label = format!("{provider}.{model_name}");
    if table.keys().any(|k| k.starts_with(LEGACY_RATE_PREFIX)) {
        out.table_errors.push(format!(
            "[agent.{label}] uses {LEGACY_RATE_PREFIX}_* keys, which were renamed to \
             usd_per_megatoken_* (same values: USD per million tokens); rename them"
        ));
    }
    if !table.keys().any(|k| k.starts_with("usd_per_megatoken")) {
        return;
    }
    match token_cost_rates_from_value(model_val) {
        Ok(rates) => {
            out.rates.insert(format!("{provider}:{model_name}"), rates);
        }
        Err(e) => out.table_errors.push(format!("[agent.{label}]: {e}")),
    }
}

pub(crate) fn token_cost_rates_from_value(value: &toml::Value) -> Result<TokenCostRates, String> {
    Ok(TokenCostRates {
        usd_per_megatoken_in: non_negative_rate(
            value.get("usd_per_megatoken_in"),
            "usd_per_megatoken_in",
        )?,
        usd_per_megatoken_out: non_negative_rate(
            value.get("usd_per_megatoken_out"),
            "usd_per_megatoken_out",
        )?,
        usd_per_megatoken_cache_read: non_negative_rate(
            value.get("usd_per_megatoken_cache_read"),
            "usd_per_megatoken_cache_read",
        )?,
        usd_per_megatoken_cache_write: non_negative_rate(
            value.get("usd_per_megatoken_cache_write"),
            "usd_per_megatoken_cache_write",
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
