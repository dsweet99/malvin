//! `malvin models` listing via `pi --list-models`.

use std::time::Duration;

use super::discover::resolve_pi_bin;
use crate::command_output_timeout::{command_output_with_timeout, timeout_ms_from_env};

/// Default wall-clock budget for `pi --list-models` (override with
/// `MALVIN_PI_LIST_MODELS_TIMEOUT_MS`).
pub const DEFAULT_PI_LIST_MODELS_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiModelListing {
    pub id: String,
    pub name: String,
    /// From the `thinking` column when present (`yes` / `no`).
    pub thinking: Option<bool>,
}

/// Resolve the `pi --list-models` wall-clock timeout.
#[must_use]
pub fn pi_list_models_timeout() -> Duration {
    timeout_ms_from_env(
        "MALVIN_PI_LIST_MODELS_TIMEOUT_MS",
        DEFAULT_PI_LIST_MODELS_TIMEOUT_MS,
    )
}

/// List models from the external `pi` CLI.
///
/// # Errors
///
/// Returns an error when `pi` is missing, `--list-models` fails, the listing
/// exceeds [`pi_list_models_timeout`], or exit 0 stdout yields no parseable rows.
pub fn list_pi_models_sync() -> Result<Vec<PiModelListing>, String> {
    let bin = resolve_pi_bin()?;
    let mut cmd = crate::malvin_sandbox::malvin_std_command(&bin);
    cmd.arg("--list-models");
    let output = command_output_with_timeout(cmd, pi_list_models_timeout(), "pi --list-models")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "pi --list-models exited with {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let models = parse_list_models_table(&stdout);
    if models.is_empty() {
        return Err(
            "pi --list-models produced no parseable model rows (empty or unrecognized table)"
                .to_string(),
        );
    }
    Ok(models)
}

fn is_separator_line(line: &str) -> bool {
    line.chars()
        .all(|c| c == '-' || c == ' ' || c == '\t' || c == '–')
}

fn is_provider_id(provider: &str) -> bool {
    !provider.is_empty()
        && provider
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn is_noise_line(line: &str, lower: &str) -> bool {
    (lower.starts_with("provider") && lower.contains("model"))
        || is_separator_line(line)
        || lower.starts_with("showing ")
        || lower.starts_with("run `")
}

/// Column starts from a `pi --list-models` header.
#[derive(Debug, Clone, Copy)]
struct HeaderColumns {
    model_start: usize,
    context_start: usize,
    thinking_start: Option<usize>,
    thinking_end: Option<usize>,
}

fn header_columns(header: &str) -> Option<HeaderColumns> {
    let lower = header.to_ascii_lowercase();
    let provider = lower.find("provider")?;
    let model = lower.find("model")?;
    let context = lower.find("context")?;
    if !(provider < model && model < context) {
        return None;
    }
    let thinking_start = lower.find("thinking");
    let thinking_end = thinking_start.and_then(|start| {
        // Next column after thinking (usually `images`), else end of line.
        let after = &lower[start + "thinking".len()..];
        let rel = after.find(|c: char| c.is_ascii_alphabetic())?;
        Some(start + "thinking".len() + rel)
    });
    Some(HeaderColumns {
        model_start: model,
        context_start: context,
        thinking_start,
        thinking_end,
    })
}

fn listing_from_fixed_columns(line: &str, cols: HeaderColumns) -> Option<PiModelListing> {
    if line.len() < cols.context_start || cols.model_start >= cols.context_start {
        return None;
    }
    let provider = line.get(..cols.model_start)?.trim();
    let model = line.get(cols.model_start..cols.context_start)?.trim();
    if model.is_empty() || !is_provider_id(provider) {
        return None;
    }
    Some(PiModelListing {
        name: model.to_string(),
        id: format!("{provider}/{model}"),
        thinking: thinking_from_fixed_columns(line, cols),
    })
}

fn thinking_from_fixed_columns(line: &str, cols: HeaderColumns) -> Option<bool> {
    let start = cols.thinking_start?;
    let end = cols.thinking_end.unwrap_or(line.len()).min(line.len());
    if start >= end || start >= line.len() {
        return None;
    }
    parse_thinking_cell(line.get(start..end)?.trim())
}

fn parse_thinking_cell(cell: &str) -> Option<bool> {
    match cell.to_ascii_lowercase().as_str() {
        "yes" | "y" | "true" => Some(true),
        "no" | "n" | "false" => Some(false),
        _ => None,
    }
}

fn listing_from_whitespace_row(line: &str) -> Option<PiModelListing> {
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 2 {
        return None;
    }
    let provider = cols[0];
    let model = cols[1];
    if model.is_empty() || !is_provider_id(provider) {
        return None;
    }
    // Live tables: provider model context max-out thinking images
    let thinking = cols.iter().find_map(|c| parse_thinking_cell(c));
    Some(PiModelListing {
        name: model.to_string(),
        id: format!("{provider}/{model}"),
        thinking,
    })
}

#[must_use]
pub(crate) fn parse_list_models_table(text: &str) -> Vec<PiModelListing> {
    let mut out = Vec::new();
    let mut columns: Option<HeaderColumns> = None;
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if is_noise_line(line, &lower) {
            if columns.is_none() {
                columns = header_columns(line);
            }
            continue;
        }
        let row = columns.map_or_else(
            || listing_from_whitespace_row(line),
            |cols| {
                listing_from_fixed_columns(line, cols).or_else(|| listing_from_whitespace_row(line))
            },
        );
        if let Some(row) = row {
            out.push(row);
        }
    }
    out
}
