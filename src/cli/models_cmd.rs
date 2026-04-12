//! `malvin models` — list models via the Cursor agent CLI.

use std::path::PathBuf;
use std::process::Command;

use clap::Args;

use malvin::env_path::agent_or_cursor_agent_bin;

use super::shared_opts::DEFAULT_CLI_MODEL;

/// Arguments for [`run_models`].
#[derive(Args, Debug)]
pub struct ModelsArgs {}

fn resolve_models_cli() -> Result<PathBuf, String> {
    agent_or_cursor_agent_bin().ok_or_else(|| {
        "Neither `agent` nor `cursor-agent` was found on PATH. Install the Cursor CLI agent to use `malvin models`."
            .to_string()
    })
}

fn consume_csi_sequence(chars: &mut std::iter::Peekable<std::str::Chars>) {
    // CSI ends with a final byte in 0x40..=0x7E (ECMA-48), not only SGR `m`.
    for ch in chars.by_ref() {
        if matches!(ch, '\x40'..='\x7e') {
            break;
        }
    }
}

fn consume_osc_sequence(chars: &mut std::iter::Peekable<std::str::Chars>) {
    // OSC (window title, hyperlinks, …): BEL or ST (ESC \).
    while let Some(ch) = chars.next() {
        if ch == '\x07' {
            return;
        }
        if ch == '\x1b' && chars.peek() == Some(&'\\') {
            chars.next();
            return;
        }
    }
}

fn strip_ansi_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.peek().copied() {
            Some('[') => {
                chars.next();
                consume_csi_sequence(&mut chars);
            }
            Some(']') => {
                chars.next();
                consume_osc_sequence(&mut chars);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Print models from `cursor-agent models` / `agent models` with a short footer.
pub fn run_models() -> Result<(), String> {
    let bin = resolve_models_cli()?;
    let output = Command::new(&bin)
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
    println!();
    println!("Default model in malvin: {DEFAULT_CLI_MODEL}");
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


fn print_parsed_or_fallback(text: &str) {
    let mut printed = false;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some((name, rest)) = parse_model_line(t) {
            println!("{name}\t{rest}");
            printed = true;
        }
    }
    if !printed {
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
    }
}

/// Best-effort parse: `name — description`, `name - description`, or two-column spacing.
fn parse_model_line(line: &str) -> Option<(&str, String)> {
    if let Some((a, b)) = line.split_once(" — ") {
        return Some((a.trim(), b.trim().to_string()));
    }
    if let Some((a, b)) = line.split_once(" - ") {
        if a.split_whitespace().count() <= 4 && !b.trim().is_empty() {
            return Some((a.trim(), b.trim().to_string()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_csi_color() {
        let s = "\x1b[31mfoo\x1b[0m";
        assert_eq!(strip_ansi_escapes(s), "foo");
    }

    #[test]
    fn strip_ansi_preserves_following_text_when_csi_does_not_end_with_m() {
        // CSI sequences may end in letters other than `m` (e.g. erase display).
        assert_eq!(strip_ansi_escapes("foo\x1b[2Jbar"), "foobar");
    }

    #[test]
    fn strip_ansi_removes_osc_set_window_title() {
        // OSC sequences use ESC ] ... BEL (or ST), not CSI ESC [ ... Final.
        let raw = "\x1b]0;cursor-agent\x07composer-2 — default";
        assert_eq!(
            strip_ansi_escapes(raw),
            "composer-2 — default",
            "OSC title/hyperlink noise should be stripped for stable `models` parsing"
        );
    }

    #[test]
    fn strip_ansi_removes_osc_terminated_with_st() {
        let raw = "x\x1b]52;c;Z\x1b\\y";
        assert_eq!(strip_ansi_escapes(raw), "xy", "OSC may end with ST (ESC \\) instead of BEL");
    }

    #[test]
    fn trim_trailing_tips_drops_banner() {
        let t = "a\nb\nTip: upgrade\n";
        assert_eq!(trim_trailing_tip_lines(t).lines().count(), 2);
    }

    #[test]
    fn trim_trailing_tips_drops_tip_space_banner_without_colon() {
        let t = "a\nb\ntip use TLS in prod\n";
        assert_eq!(trim_trailing_tip_lines(t).lines().count(), 2);
    }

    #[test]
    fn trim_trailing_tips_keeps_last_line_that_mentions_tip_mid_sentence() {
        let t = "composer-2 — Fast\nSee tip: use TLS in prod\n";
        assert_eq!(
            trim_trailing_tip_lines(t),
            "composer-2 — Fast\nSee tip: use TLS in prod"
        );
    }

    #[test]
    fn trim_trailing_tips_keeps_line_starting_with_tip_of_english_phrase() {
        let t = "composer-2 — Fast\nTip of the iceberg — latency matters\n";
        assert_eq!(
            trim_trailing_tip_lines(t),
            "composer-2 — Fast\nTip of the iceberg — latency matters"
        );
    }

    #[test]
    fn parse_model_line_splits_em_dash() {
        let (n, d) = parse_model_line("composer-2 — Fast").expect("parse");
        assert_eq!(n, "composer-2");
        assert_eq!(d, "Fast");
    }

    #[test]
    fn kiss_stringify_models_cmd() {
        let _ = stringify!(ModelsArgs);
        let _ = stringify!(run_models);
        let _ = stringify!(resolve_models_cli);
        let _ = stringify!(consume_csi_sequence);
        let _ = stringify!(consume_osc_sequence);
        let _ = stringify!(strip_ansi_escapes);
        let _ = stringify!(trim_trailing_tip_lines);
        let _ = stringify!(looks_like_tip_banner_line);
        let _ = stringify!(print_parsed_or_fallback);
        let _ = stringify!(parse_model_line);
    }
}
