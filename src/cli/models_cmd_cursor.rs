use std::path::PathBuf;
use std::process::Output;
use std::time::Duration;

use crate::agent_or_cursor_agent_bin;
use crate::ansi_strip::strip_ansi_escapes;
use crate::command_output_timeout::{command_output_with_timeout, timeout_ms_from_env};
use crate::model_id::CURSOR_PREFIX;
use crate::output::{MALVIN_WHO, print_stdout_line};

use super::line_matches_prefix;
use super::models_cmd_parse::{print_parsed_or_fallback_prefixed, trim_trailing_tip_lines};

pub const DEFAULT_CURSOR_LIST_MODELS_TIMEOUT_MS: u64 = 30_000;

#[must_use]
pub fn cursor_list_models_timeout() -> Duration {
    timeout_ms_from_env(
        "MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS",
        DEFAULT_CURSOR_LIST_MODELS_TIMEOUT_MS,
    )
}

pub(super) fn print_cursor_models(filter: Option<&str>) -> Result<(), String> {
    if print_cursor_models_via_sdk(filter).is_ok() {
        return Ok(());
    }
    print_cursor_models_via_cli(filter)
}

fn print_cursor_models_via_sdk(filter: Option<&str>) -> Result<(), String> {
    let output = run_cursor_sdk_models_js()?;
    let raw = String::from_utf8_lossy(&output.stdout);
    if !sdk_catalog_has_model_rows(&raw) {
        return Err("cursor SDK models returned an empty catalog".to_string());
    }
    let rows = sdk_model_rows_from_stdout(&raw);
    print_filtered_model_rows(&rows, filter);
    Ok(())
}

pub(super) fn sdk_catalog_has_model_rows(raw: &str) -> bool {
    raw.lines().any(|line| {
        let t = line.trim();
        !t.is_empty() && !t.starts_with('{')
    })
}

fn run_cursor_sdk_models_js() -> Result<Output, String> {
    let models_js = crate::cursor_sdk::bridge_path::resolve_models_js()?;
    let node = crate::cursor_sdk::node_resolve::resolve_node_bin()?;
    let mut cmd = crate::malvin_sandbox::malvin_std_command(&node);
    crate::cursor_sdk::node_resolve::apply_quiet_node_cli_std(&mut cmd);
    cmd.arg(&models_js);
    let output =
        command_output_with_timeout(cmd, cursor_list_models_timeout(), "cursor SDK models")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cursor SDK models failed: {}", stderr.trim()));
    }
    Ok(output)
}

pub(super) fn sdk_model_rows_from_stdout(raw: &str) -> Vec<String> {
    let mut rows = Vec::new();
    let mut saw_auto = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('{') {
            continue;
        }
        let id = t.split('\t').next().unwrap_or(t).trim();
        if id == "cursor:auto" {
            saw_auto = true;
        }
        rows.push(t.to_string());
    }
    if !saw_auto {
        rows.insert(0, format!("{CURSOR_PREFIX}auto"));
    }
    rows
}

fn print_filtered_model_rows(rows: &[String], filter: Option<&str>) {
    for row in rows {
        if line_matches_prefix(row, filter) {
            print_stdout_line(MALVIN_WHO, row);
        }
    }
}

pub(super) fn resolve_models_cli() -> Result<PathBuf, String> {
    agent_or_cursor_agent_bin().ok_or_else(|| {
        "Neither `agent` nor `cursor-agent` was found on PATH. Install the Cursor CLI agent to list models (`malvin admin models`)."
            .to_string()
    })
}

pub(super) fn print_cursor_models_via_cli(filter: Option<&str>) -> Result<(), String> {
    let bin = resolve_models_cli()?;
    let mut cmd = crate::malvin_sandbox::malvin_std_command(&bin);
    cmd.arg("models");
    let output = command_output_with_timeout(
        cmd,
        cursor_list_models_timeout(),
        &format!("{} models", bin.display()),
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        let detail = if msg.is_empty() {
            format!("`{} models` exited with {}", bin.display(), output.status)
        } else {
            format!("`{} models` failed: {msg}", bin.display())
        };
        return Err(format!("agent models failed: {detail}"));
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let text = strip_ansi_escapes(raw.as_ref());
    let cleaned = trim_trailing_tip_lines(&text);
    print_parsed_or_fallback_prefixed(&cleaned, CURSOR_PREFIX, filter);
    Ok(())
}
