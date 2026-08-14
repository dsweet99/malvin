
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

pub(super) fn looks_like_tip_banner_line(lowercase_trimmed: &str) -> bool {
    if lowercase_trimmed.starts_with("tip:") {
        return true;
    }
    if let Some(after_tip_space) = lowercase_trimmed.strip_prefix("tip ") {
        return !after_tip_space.starts_with("of ");
    }
    false
}

#[cfg(test)]
pub(super) fn models_display_lines(text: &str, prefix: &str) -> Option<Vec<String>> {
    models_display_lines_filtered(text, prefix, None)
}

pub(super) fn models_display_lines_filtered(
    text: &str,
    prefix: &str,
    filter: Option<&str>,
) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || is_non_model_banner_line(t) {
            continue;
        }
        let row = if let Some((name, rest)) = parse_model_line(t) {
            format!("{prefix}{name}\t{rest}")
        } else {
            format!("{prefix}{t}")
        };
        if super::line_matches_prefix(&row, filter) {
            out.push(row);
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

pub(super) fn is_non_model_banner_line(line: &str) -> bool {
    let low = line.trim().to_ascii_lowercase();
    low == "available models" || low.starts_with("no models")
}

pub(super) fn print_parsed_or_fallback_prefixed(
    text: &str,
    prefix: &str,
    filter: Option<&str>,
) {
    match models_display_lines_filtered(text, prefix, filter) {
        Some(lines) => {
            for line in lines {
                print_stdout_line(MALVIN_WHO, &line);
            }
        }
        None => {
            if filter.is_none() {
                print_stdout_text(MALVIN_WHO, text);
            }
        }
    }
}

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

#[cfg(test)]
mod banner_tests {
    use super::{is_non_model_banner_line, models_display_lines};

    #[test]
    fn models_display_lines_skips_available_models_header() {
        let text = "Available models\n\nauto - Auto (current, default)\n";
        let lines = models_display_lines(text, "").expect("non-empty");
        assert_eq!(
            lines,
            vec!["auto\tAuto (current, default)".to_string()],
            "header must not become a fake model row: {lines:?}"
        );
        assert!(
            !lines.iter().any(|l| l.to_ascii_lowercase().contains("available")),
            "must not keep Available header: {lines:?}"
        );
        assert!(is_non_model_banner_line("Available models"));
    }

    #[test]
    fn models_display_lines_skips_no_models_available_status() {
        let text = "No models available for this account.\n";
        assert!(
            models_display_lines(text, "cursor:").is_none(),
            "status line must not become a model row"
        );
        let mixed = "No models available for this account.\nauto - Auto (current, default)\n";
        let lines = models_display_lines(mixed, "cursor:").expect("keep real rows");
        assert_eq!(
            lines,
            vec!["cursor:auto\tAuto (current, default)".to_string()]
        );
        assert!(
            !lines.iter().any(|l| l.starts_with("cursor:No")),
            "must not keep No-models status as a row: {lines:?}"
        );
        assert!(is_non_model_banner_line(
            "No models available for this account."
        ));
    }
}
