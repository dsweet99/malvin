use std::fmt::Write as _;

use super::types::{
    ansi_tool_coral, ansi_tool_dark, ansi_tool_teal, ANSI_BOLD, ANSI_DIM, ANSI_RESET,
};

const DONE_VERB_PREFIXES: &[&str] = &["Read ", "Edit ", "Search ", "Run "];

pub fn tool_summary_stdout_display(plain: &str) -> String {
    if !crate::output::stdout_use_color() {
        return plain.to_string();
    }
    apply_tool_summary_ansi(plain)
}

pub(crate) fn split_outer_brackets(plain: &str) -> (&str, &str, &str) {
    plain
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .map_or(("", plain, ""), |inner| ("[", inner, "]"))
}

fn dark_bracket(bracket: &str) -> String {
    format!("{}{bracket}{ANSI_RESET}", ansi_tool_dark())
}

pub(crate) fn apply_tool_summary_ansi(plain: &str) -> String {
    let (open, inner, close) = split_outer_brackets(plain);
    let mut out = if open.is_empty() {
        String::new()
    } else {
        dark_bracket(open)
    };
    let mut rest = inner;
    while let Some(idx) = rest.find('·') {
        let (left, right) = rest.split_at(idx);
        out.push_str(&ansi_style_tool_segment(left));
        let _ = write!(out, "{}·{ANSI_RESET}", ansi_tool_teal());
        rest = right.trim_start_matches('·').trim_start();
    }
    out.push_str(&ansi_style_tool_segment(rest));
    if !close.is_empty() {
        out.push_str(&dark_bracket(close));
    }
    out
}

pub(crate) fn ansi_style_tool_segment(seg: &str) -> String {
    let seg = seg.trim();
    if seg.is_empty() {
        return String::new();
    }
    if seg.contains('✓') {
        return seg.replace('✓', &format!("{}✓{ANSI_RESET}", ansi_tool_teal()));
    }
    if seg.contains('✗') {
        return seg.replace('✗', &format!("{}✗{ANSI_RESET}", ansi_tool_coral()));
    }
    ansi_style_tool_segment_running_or_path(seg)
}

pub(crate) fn tool_line_colon_prefix(seg: &str) -> (&str, &str) {
    if let Some(rest) = seg.strip_prefix(":: ") {
        return (":: ", rest);
    }
    if let Some(inner) = seg.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        return ("[", inner);
    }
    ("", seg)
}

pub(crate) fn ansi_style_dark_verb(verb: &str) -> String {
    format!("{ANSI_BOLD}{}{verb}{ANSI_RESET}", ansi_tool_dark())
}

pub(crate) fn ansi_style_running_verb(seg: &str) -> String {
    let (colon, body) = tool_line_colon_prefix(seg);
    let verb_end = body.find(' ').unwrap_or(body.len());
    let (verb, tail) = body.split_at(verb_end);
    format!("{colon}{}{}", ansi_style_dark_verb(verb), ansi_style_path_tail(tail))
}

pub(crate) fn ansi_style_done_verb(seg: &str) -> String {
    let (colon, body) = tool_line_colon_prefix(seg);
    for prefix in DONE_VERB_PREFIXES {
        if let Some(tail) = body.strip_prefix(prefix) {
            let verb = prefix.trim_end();
            let mut out = format!("{colon}{}", ansi_style_dark_verb(verb));
            if !tail.is_empty() {
                out.push(' ');
                out.push_str(&ansi_style_path_tail(tail));
            }
            return out;
        }
    }
    if body_is_search_done_verb(body) {
        let tail = body.strip_prefix("Search").unwrap_or(body).trim_start();
        let mut out = format!("{colon}{}", ansi_style_dark_verb("Search"));
        if !tail.is_empty() {
            out.push(' ');
            out.push_str(&ansi_style_path_tail(tail));
        }
        return out;
    }
    format!("{colon}{}", ansi_style_path_tail(body))
}

fn body_is_search_done_verb(body: &str) -> bool {
    body.strip_prefix("Search").is_some_and(|rest| {
        rest.is_empty() || rest.starts_with(' ') || rest.starts_with('·')
    })
}

pub(crate) fn ansi_style_tool_segment_running_or_path(seg: &str) -> String {
    let (_, body) = tool_line_colon_prefix(seg);
    if body.ends_with('…')
        || body.starts_with("Reading ")
        || body.starts_with("Run ")
        || body.starts_with("Editing ")
        || body.starts_with("Searching")
    {
        return ansi_style_running_verb(seg);
    }
    if body.starts_with("Read ")
        || body.starts_with("Edit ")
        || body_is_search_done_verb(body)
    {
        return ansi_style_done_verb(seg);
    }
    if seg.contains("matches") || seg.contains("exit ") || seg.contains("ms") || seg.contains('s')
    {
        return format!("{ANSI_DIM}{seg}{ANSI_RESET}");
    }
    ansi_style_path_tail(seg)
}

pub(crate) fn is_byte_size_segment(seg: &str) -> bool {
    seg.ends_with(" B") || seg.ends_with(" KB") || seg.ends_with(" MB")
}

pub(crate) fn ansi_style_path_tail(seg: &str) -> String {
    if is_byte_size_segment(seg) {
        return format!("{ANSI_DIM}{seg}{ANSI_RESET}");
    }
    format!("{}{seg}{ANSI_RESET}", ansi_tool_teal())
}

#[cfg(test)]
mod inline_tests {
    use super::{apply_tool_summary_ansi, body_is_search_done_verb};

    #[test]
    fn body_is_search_done_verb_covers_bare_and_spaced_forms() {
        assert!(body_is_search_done_verb("Search"));
        assert!(body_is_search_done_verb("Search "));
        assert!(body_is_search_done_verb("Search · matches"));
        assert!(body_is_search_done_verb("Search needle · 1ms"));
        assert!(!body_is_search_done_verb("Searching"));
        assert!(!body_is_search_done_verb("Research"));
    }

    #[test]
    fn bracket_wrapped_search_done_uses_dark_brackets() {
        let styled = apply_tool_summary_ansi("[Search · matches]");
        assert!(styled.contains('[') && styled.contains(']'));
        assert!(styled.contains("Search"));
    }
}

#[cfg(test)]
#[path = "ansi_tests.rs"]
mod ansi_tests;
