//! Prime model listing for `malvin models`.

use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimeModelListing {
    pub id: String,
    pub name: String,
}

/// Best-effort list of `prime:<provider>/<model>` rows (full catalog from bridge or CLI).
///
/// # Errors
///
/// Returns an error when the models script / CLI cannot be run.
pub fn list_prime_models_sync() -> Result<Vec<PrimeModelListing>, String> {
    if let Ok(from_bridge) = list_via_models_js() {
        if !from_bridge.is_empty() {
            return Ok(from_bridge);
        }
    }
    list_via_prime_agent_cli()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_prime_model_lines_keeps_full_catalog() {
        use std::fmt::Write;
        let mut text = String::new();
        for i in 0..20 {
            let _ = writeln!(text, "prime:openai/m{i}");
            let _ = writeln!(text, "prime:openrouter/org/m{i}");
        }
        let rows = parse_prime_model_lines(&text);
        assert_eq!(rows.len(), 40);
        assert!(rows.iter().all(|r| r.id != "…"));
        assert!(!rows.iter().any(|r| r.name.contains("more via")));
    }

    #[test]
    fn parse_prime_agent_table_skips_header() {
        let text = "provider    model\nopenai      gpt-4o\nopenrouter  anthropic/claude-3-haiku\n";
        let rows = parse_prime_agent_table(text);
        assert_eq!(rows[0].id, "openai/gpt-4o");
        assert_eq!(rows[1].id, "openrouter/anthropic/claude-3-haiku");
    }
}
