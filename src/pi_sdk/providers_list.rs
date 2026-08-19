use std::collections::HashMap;

use super::auth::env_nonempty;
use super::discover::resolve_pi_bin;
use super::models_list::pi_list_models_timeout;
use crate::command_output_timeout::command_output_with_timeout;

pub fn list_pi_provider_auth_sync() -> Result<HashMap<String, Vec<String>>, String> {
    let bin = resolve_pi_bin()?;
    let mut cmd = crate::malvin_sandbox::malvin_std_command(&bin);
    cmd.arg("--list-providers");
    let output = command_output_with_timeout(cmd, pi_list_models_timeout(), "pi --list-providers")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "pi --list-providers exited with {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let map = parse_list_providers_table(&stdout);
    if map.is_empty() {
        return Err(
            "pi --list-providers produced no parseable provider rows (empty or unrecognized table)"
                .to_string(),
        );
    }
    Ok(map)
}

#[must_use]
pub fn provider_authenticated_from_map(provider: &str, map: &HashMap<String, Vec<String>>) -> bool {
    match map.get(provider) {
        None => true,
        Some(keys) if keys.is_empty() => true,
        Some(keys) => keys.iter().any(|k| env_nonempty(k)),
    }
}

#[must_use]
pub(crate) fn parse_list_providers_table(text: &str) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    let mut columns = None;
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() || is_dash_row(line) || is_providers_noise_line(line) {
            continue;
        }
        if columns.is_none() {
            columns = providers_header_columns(line);
            if columns.is_some() {
                continue;
            }
        }
        if let Some(cols) = columns {
            record_provider_row(&mut map, line, cols);
        }
    }
    map
}

fn is_dash_row(line: &str) -> bool {
    line.chars()
        .all(|c| c == '-' || c == ' ' || c == '\t' || c == '–')
}

fn is_providers_noise_line(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    lower.starts_with("showing ") || lower.contains("providers available")
}

#[derive(Clone, Copy)]
struct ProviderColumns {
    name: usize,
    aliases: usize,
    auth: usize,
    api: usize,
}

fn providers_header_columns(header: &str) -> Option<ProviderColumns> {
    let lower = header.to_ascii_lowercase();
    let provider = lower.find("provider")?;
    let name = lower.find("name")?;
    let aliases = lower.find("aliases")?;
    let auth = lower.find("auth env")?;
    let api = lower.rfind("api")?;
    if provider < name && name < aliases && aliases < auth && auth < api {
        Some(ProviderColumns {
            name,
            aliases,
            auth,
            api,
        })
    } else {
        None
    }
}

fn record_provider_row(map: &mut HashMap<String, Vec<String>>, line: &str, cols: ProviderColumns) {
    let provider = col(line, 0, cols.name);
    let looks_like_id = !provider.is_empty()
        && provider
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if !looks_like_id {
        return;
    }
    let aliases = col(line, cols.aliases, cols.auth);
    let keys = auth_env_keys_from_cell(col(line, cols.auth, cols.api));
    map.insert(provider.to_string(), keys.clone());
    for alias in aliases.split(',') {
        let alias = alias.trim();
        if !alias.is_empty() && alias != "-" {
            map.insert(alias.to_string(), keys.clone());
        }
    }
}

fn col(line: &str, start: usize, end: usize) -> &str {
    let n = line.len();
    if start >= n || end > n || !line.is_char_boundary(start) || !line.is_char_boundary(end.min(n))
    {
        return "";
    }
    line[start..end.min(n)].trim()
}

fn auth_env_keys_from_cell(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|s| is_auth_env_key(s))
        .map(ToString::to_string)
        .collect()
}

fn is_auth_env_key(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
#[path = "providers_list_tests.rs"]
mod providers_list_tests;
