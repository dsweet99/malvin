use std::process::Command;

use super::pi_backend::discover::{resolve_npm_pi_cli_entry, resolve_npm_pi_entry};
use super::pi_backend::models_rpc::BASE_THINKING_LEVELS;

pub fn refresh_npm_pi_models() -> Result<(), String> {
    let entry = resolve_npm_pi_cli_entry()?;
    let node = crate::backends::cursor_sdk::node_resolve::resolve_node_bin()?;
    let output = Command::new(node)
        .arg(&entry)
        .arg("update")
        .arg("--models")
        .output()
        .map_err(|e| format!("spawn npm pi update --models: {e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        let msg = if err.trim().is_empty() { out } else { err };
        return Err(format!(
            "npm pi update --models failed: {}",
            msg.trim().chars().take(240).collect::<String>()
        ));
    }
    Ok(())
}

pub fn list_npm_pi_display_models() -> Result<Vec<(String, String)>, String> {
    super::pi_backend::models_rpc::list_rpc_models().or_else(|_| list_table_models())
}

fn list_table_models() -> Result<Vec<(String, String)>, String> {
    let entry = resolve_npm_pi_entry()?;
    let node = crate::backends::cursor_sdk::node_resolve::resolve_node_bin()?;
    let output = Command::new(node)
        .arg(&entry)
        .arg("--list-models")
        .output()
        .map_err(|e| format!("spawn npm pi --list-models: {e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "npm pi --list-models failed: {}",
            err.trim().chars().take(240).collect::<String>()
        ));
    }
    Ok(parse_list_models_table(&list_models_table_text(
        &output.stdout,
        &output.stderr,
    )))
}

fn list_models_table_text(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout);
    if stdout.trim().is_empty() {
        String::from_utf8_lossy(stderr).into_owned()
    } else {
        stdout.into_owned()
    }
}

fn parse_list_models_table(stdout: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with("provider") {
            continue;
        }
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 2 {
            continue;
        }
        let provider = cols[0];
        let model = cols[1];
        if provider.is_empty() || model.is_empty() {
            continue;
        }
        let id = format!("{provider}/{model}");
        let levels = if cols.get(4).copied() == Some("yes") {
            BASE_THINKING_LEVELS.join("|")
        } else {
            "off".to_string()
        };
        out.push((id, format!("{model}\tthinking={levels}")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_table_rows() {
        let sample = "\
provider    model        context  max-out  thinking  images
openai      gpt-4o       128K     16.4K    no        yes
anthropic   claude-4     200K     32K      yes       yes
";
        let rows = parse_list_models_table(sample);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "openai/gpt-4o");
        assert_eq!(rows[0].1, "gpt-4o\tthinking=off");
        assert_eq!(rows[1].0, "anthropic/claude-4");
        assert_eq!(rows[1].1, "claude-4\tthinking=off|minimal|low|medium|high");
    }

    #[test]
    fn list_models_table_text_falls_back_to_stderr() {
        let from_stdout =
            list_models_table_text(b"provider model\nopenai gpt-4o 1 1 no yes\n", b"noise\n");
        assert!(from_stdout.contains("openai"));
        let from_stderr =
            list_models_table_text(b"  \n", b"provider model\nopenai gpt-4o 1 1 no yes\n");
        assert!(from_stderr.contains("openai"));
    }
}
