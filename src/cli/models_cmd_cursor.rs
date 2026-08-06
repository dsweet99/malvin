//! Cursor model listing for `malvin models`.

use std::path::PathBuf;

use crate::agent_or_cursor_agent_bin;
use crate::ansi_strip::strip_ansi_escapes;
use crate::model_id::CURSOR_PREFIX;
use crate::output::{MALVIN_WHO, print_stdout_line};

use super::line_matches_prefix;
use super::models_cmd_parse::{print_parsed_or_fallback_prefixed, trim_trailing_tip_lines};

pub(super) fn print_cursor_models(filter: Option<&str>) -> Result<(), String> {
    if print_cursor_models_via_sdk(filter).is_ok() {
        return Ok(());
    }
    print_cursor_models_via_cli(filter)
}

fn print_cursor_models_via_sdk(filter: Option<&str>) -> Result<(), String> {
    let models_js = crate::cursor_sdk::bridge_path::resolve_models_js()?;
    let node = crate::cursor_sdk::node_resolve::resolve_node_bin()?;
    let mut cmd = crate::malvin_sandbox::malvin_std_command(&node);
    crate::cursor_sdk::node_resolve::apply_quiet_node_cli_std(&mut cmd);
    let output = cmd
        .arg(&models_js)
        .output()
        .map_err(|e| format!("failed to execute cursor SDK models: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cursor SDK models failed: {}", stderr.trim()));
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('{') {
            continue;
        }
        if line_matches_prefix(t, filter) {
            print_stdout_line(MALVIN_WHO, t);
        }
    }
    Ok(())
}

pub(super) fn resolve_models_cli() -> Result<PathBuf, String> {
    agent_or_cursor_agent_bin().ok_or_else(|| {
        "Neither `agent` nor `cursor-agent` was found on PATH. Install the Cursor CLI agent to list models (`malvin models`)."
            .to_string()
    })
}

fn print_cursor_models_via_cli(filter: Option<&str>) -> Result<(), String> {
    let bin = resolve_models_cli()?;
    let output = crate::malvin_sandbox::malvin_std_command(&bin)
        .arg("models")
        .output()
        .map_err(|e| format!("failed to execute {} models: {e}", bin.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        let detail = if msg.is_empty() {
            format!("`{} models` exited with {}", bin.display(), output.status)
        } else {
            format!("`{} models` failed: {msg}", bin.display())
        };
        return Err(detail);
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let text = strip_ansi_escapes(raw.as_ref());
    let cleaned = trim_trailing_tip_lines(&text);
    print_parsed_or_fallback_prefixed(&cleaned, CURSOR_PREFIX, filter);
    Ok(())
}
