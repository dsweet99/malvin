//! `malvin models` listing via `pi --list-models`.

use std::process::Command;

use super::discover::resolve_pi_bin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiModelListing {
    pub id: String,
    pub name: String,
}

/// List models from the external `pi` CLI.
///
/// # Errors
///
/// Returns an error when `pi` is missing or `--list-models` fails.
pub fn list_pi_models_sync() -> Result<Vec<PiModelListing>, String> {
    let bin = resolve_pi_bin()?;
    let output = Command::new(&bin)
        .arg("--list-models")
        .output()
        .map_err(|e| format!("pi --list-models failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "pi --list-models exited with {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_list_models_table(&stdout))
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

fn listing_from_row(line: &str) -> Option<PiModelListing> {
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 2 {
        return None;
    }
    let provider = cols[0];
    let model = cols[1];
    if model.is_empty() || !is_provider_id(provider) {
        return None;
    }
    Some(PiModelListing {
        name: model.to_string(),
        id: format!("{provider}/{model}"),
    })
}

#[must_use]
pub(crate) fn parse_list_models_table(text: &str) -> Vec<PiModelListing> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if is_noise_line(line, &lower) {
            continue;
        }
        if let Some(row) = listing_from_row(line) {
            out.push(row);
        }
    }
    out
}
