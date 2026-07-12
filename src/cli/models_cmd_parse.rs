//! Parse and display helpers for `malvin models`.

use crate::output::{MALVIN_WHO, print_stdout_line, print_stdout_text};

pub(super) fn trim_trailing_tip_lines(text: &str) -> String {
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

/// Trailing banners from `cursor-agent models` look like `Tip: …` or `tip …` (space form).
pub(super) fn looks_like_tip_banner_line(lowercase_trimmed: &str) -> bool {
    if lowercase_trimmed.starts_with("tip:") {
        return true;
    }
    if let Some(after_tip_space) = lowercase_trimmed.strip_prefix("tip ") {
        // "Tip of the iceberg — …" is description text, not a `Tip` banner line.
        return !after_tip_space.starts_with("of ");
    }
    false
}

pub(super) fn models_display_lines(text: &str, prefix: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some((name, rest)) = parse_model_line(t) {
            out.push(format!("{prefix}{name}\t{rest}"));
        } else {
            out.push(format!("{prefix}{t}"));
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

pub(super) fn print_parsed_or_fallback_prefixed(text: &str, prefix: &str) {
    match models_display_lines(text, prefix) {
        Some(lines) => {
            for line in lines {
                print_stdout_line(MALVIN_WHO, &line);
            }
        }
        None => print_stdout_text(MALVIN_WHO, text),
    }
}

/// Best-effort parse: `name — description`, `name - description`, or two-column spacing.
pub(super) fn parse_model_line(line: &str) -> Option<(&str, String)> {
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
