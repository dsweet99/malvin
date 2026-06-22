//! `malvin models` — list models via the Cursor agent CLI.

use std::path::PathBuf;

use crate::agent_or_cursor_agent_bin;
use crate::ansi_strip::strip_ansi_escapes;
use crate::output::{MALVIN_WHO, print_stdout_line, print_stdout_text};
use clap::Args;

use crate::config::DEFAULT_CLI_MODEL;

#[derive(Args, Debug, Clone, Copy)]
pub struct ModelsArgs {}

pub(crate) const fn models_args_marker(args: ModelsArgs) -> &'static str {
    let ModelsArgs {} = std::hint::black_box(args);
    "models"
}

fn resolve_models_cli() -> Result<PathBuf, String> {
    agent_or_cursor_agent_bin().ok_or_else(|| {
        "Neither `agent` nor `cursor-agent` was found on PATH. Install the Cursor CLI agent to use `malvin models`."
            .to_string()
    })
}

/// Print models from `cursor-agent models` / `agent models` with a short footer.
pub fn run_models() -> Result<(), String> {
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
    print_parsed_or_fallback(&cleaned);
    print_stdout_line(MALVIN_WHO, "");
    print_stdout_line(
        MALVIN_WHO,
        &format!("Default model: {DEFAULT_CLI_MODEL}"),
    );
    Ok(())
}

fn trim_trailing_tip_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut end = lines.len();
    while end > 0 {
        let low = lines[end - 1].trim().to_ascii_lowercase();
        if low.is_empty() || looks_like_tip_banner_line(&low) {
            end -= 1;
        } else {
            break;
        }
    }
    lines[..end].join("\n")
}

/// Trailing banners from `cursor-agent models` look like `Tip: …` or `tip …` (space form), not
/// arbitrary prose that mentions `tip:` mid-sentence.
fn looks_like_tip_banner_line(lowercase_trimmed: &str) -> bool {
    if lowercase_trimmed.starts_with("tip:") {
        return true;
    }
    if let Some(after_tip_space) = lowercase_trimmed.strip_prefix("tip ") {
        // "Tip of the iceberg — …" is description text, not a `Tip` banner line.
        return !after_tip_space.starts_with("of ");
    }
    false
}

fn models_display_lines(text: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some((name, rest)) = parse_model_line(t) {
            out.push(format!("{name}\t{rest}"));
        } else {
            out.push(t.to_string());
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

fn print_parsed_or_fallback(text: &str) {
    match models_display_lines(text) {
        Some(lines) => {
            for line in lines {
                print_stdout_line(MALVIN_WHO, &line);
            }
        }
        None => print_stdout_text(MALVIN_WHO, text),
    }
}

/// Best-effort parse: `name — description`, `name - description`, or two-column spacing.
fn parse_model_line(line: &str) -> Option<(&str, String)> {
    if let Some((a, b)) = line.split_once(" — ") {
        return Some((a.trim(), b.trim().to_string()));
    }
    if let Some((a, b)) = line.split_once(" - ") {
        let a = a.trim();
        let b = b.trim();
        if !a.is_empty() && !b.is_empty() {
            return Some((a, b.to_string()));
        }
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        let name = parts[0];
        let rest = parts[1..].join(" ");
        return Some((name, rest));
    }
    None
}

pub(crate) mod test_hooks {

    pub fn trim_trailing_tip_lines(text: &str) -> String {
        super::trim_trailing_tip_lines(text)
    }

    pub fn looks_like_tip_banner_line(lowercase_trimmed: &str) -> bool {
        super::looks_like_tip_banner_line(lowercase_trimmed)
    }

    pub fn models_display_lines(text: &str) -> Option<Vec<String>> {
        super::models_display_lines(text)
    }

    pub fn print_parsed_or_fallback(text: &str) {
        super::print_parsed_or_fallback(text);
    }

    pub fn parse_model_line(line: &str) -> Option<(&str, String)> {
        super::parse_model_line(line)
    }

    pub fn resolve_models_cli() -> Result<std::path::PathBuf, String> {
        super::resolve_models_cli()
    }
}
