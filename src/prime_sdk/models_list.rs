//! Bounded Prime model listing for `malvin models`.

use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimeModelListing {
    pub id: String,
    pub name: String,
}

const MAX_OPENROUTER_SAMPLE: usize = 8;
const MAX_PER_PROVIDER: usize = 12;

/// Best-effort list of `prime:<provider>/<model>` rows (bounded; not the full catalog).
///
/// # Errors
///
/// Returns an error when the models script / CLI cannot be run.
pub fn list_prime_models_sync() -> Result<Vec<PrimeModelListing>, String> {
    if let Ok(from_bridge) = list_via_models_js() {
        if !from_bridge.is_empty() {
            return Ok(bound_listings(from_bridge));
        }
    }
    list_via_prime_agent_cli().map(bound_listings)
}

fn list_via_models_js() -> Result<Vec<PrimeModelListing>, String> {
    let models_js = super::bridge_path::prime_resolve_models_js()?;
    let node = super::node_resolve::prime_resolve_node_bin()?;
    let mut cmd = Command::new(&node);
    super::node_resolve::prime_apply_quiet_node_cli_std(&mut cmd);
    let output = cmd
        .arg(&models_js)
        .output()
        .map_err(|e| format!("spawn prime models.js: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "prime models.js failed (exit {}): {stderr}",
            output.status
        ));
    }
    Ok(parse_prime_model_lines(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn list_via_prime_agent_cli() -> Result<Vec<PrimeModelListing>, String> {
    let bin = crate::support_paths::lookup_bin_on_path("prime-agent").ok_or_else(|| {
        "prime-agent not on PATH and prime-sdk-bridge/dist/models.js missing".to_string()
    })?;
    let output = Command::new(bin)
        .args(["model", "list"])
        .output()
        .map_err(|e| format!("spawn prime-agent model list: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "prime-agent model list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(parse_prime_agent_table(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn parse_prime_model_lines(text: &str) -> Vec<PrimeModelListing> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let rest = line.strip_prefix("prime:").unwrap_or(line);
        if rest.is_empty() || !rest.contains('/') {
            continue;
        }
        out.push(PrimeModelListing {
            id: rest.to_string(),
            name: rest.to_string(),
        });
    }
    out
}

fn parse_prime_agent_table(text: &str) -> Vec<PrimeModelListing> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("provider") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(provider) = parts.next() else {
            continue;
        };
        let Some(model) = parts.next() else {
            continue;
        };
        if provider.contains('/') {
            continue;
        }
        let id = format!("{provider}/{model}");
        out.push(PrimeModelListing {
            id: id.clone(),
            name: id,
        });
    }
    out
}

fn bound_listings(rows: Vec<PrimeModelListing>) -> Vec<PrimeModelListing> {
    let mut openai = Vec::new();
    let mut openrouter = Vec::new();
    let mut other = Vec::new();
    for row in rows {
        if row.id.starts_with("openai/") {
            if openai.len() < MAX_PER_PROVIDER {
                openai.push(row);
            }
        } else if row.id.starts_with("openrouter/") {
            if openrouter.len() < MAX_OPENROUTER_SAMPLE {
                openrouter.push(row);
            }
        } else if other.len() < MAX_PER_PROVIDER {
            other.push(row);
        }
    }
    let mut out = openai;
    out.extend(other);
    out.extend(openrouter);
    if !out.is_empty() {
        // Hint row for the full catalog (not a real model id).
        out.push(PrimeModelListing {
            id: "…".into(),
            name: "more via `prime-agent model list`".into(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_bound_groups_providers() {
        let mut rows = Vec::new();
        for i in 0..20 {
            rows.push(PrimeModelListing {
                id: format!("openai/m{i}"),
                name: format!("m{i}"),
            });
            rows.push(PrimeModelListing {
                id: format!("openrouter/org/m{i}"),
                name: format!("or{i}"),
            });
        }
        let bounded = bound_listings(rows);
        let openai_n = bounded.iter().filter(|r| r.id.starts_with("openai/")).count();
        let or_n = bounded
            .iter()
            .filter(|r| r.id.starts_with("openrouter/"))
            .count();
        assert_eq!(openai_n, MAX_PER_PROVIDER);
        assert_eq!(or_n, MAX_OPENROUTER_SAMPLE);
        assert!(bounded.iter().any(|r| r.id == "…"));
    }

    #[test]
    fn parse_prime_agent_table_skips_header() {
        let text = "provider    model\nopenai      gpt-4o\nopenrouter  anthropic/claude-3-haiku\n";
        let rows = parse_prime_agent_table(text);
        assert_eq!(rows[0].id, "openai/gpt-4o");
        assert_eq!(rows[1].id, "openrouter/anthropic/claude-3-haiku");
    }
}
