use std::fmt::Write as _;

use super::types::{
    ANSI_BOLD, ANSI_DIM, ANSI_RESET, ANSI_TOOL_CORAL, ANSI_TOOL_CREAM, ANSI_TOOL_SAND,
    ANSI_TOOL_TEAL,
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

fn sand_bracket(bracket: &str) -> String {
    format!("{ANSI_TOOL_SAND}{bracket}{ANSI_RESET}")
}

pub(crate) fn apply_tool_summary_ansi(plain: &str) -> String {
    let (open, inner, close) = split_outer_brackets(plain);
    let mut out = if open.is_empty() {
        String::new()
    } else {
        sand_bracket(open)
    };
    let mut rest = inner;
    while let Some(idx) = rest.find('·') {
        let (left, right) = rest.split_at(idx);
        out.push_str(&ansi_style_tool_segment(left));
        let _ = write!(out, "{ANSI_TOOL_CREAM}·{ANSI_RESET}");
        rest = right.trim_start_matches('·').trim_start();
    }
    out.push_str(&ansi_style_tool_segment(rest));
    if !close.is_empty() {
        out.push_str(&sand_bracket(close));
    }
    out
}

pub(crate) fn ansi_style_tool_segment(seg: &str) -> String {
    let seg = seg.trim();
    if seg.is_empty() {
        return String::new();
    }
    if seg.contains('✓') {
        return seg.replace('✓', &format!("{ANSI_TOOL_TEAL}✓{ANSI_RESET}"));
    }
    if seg.contains('✗') {
        return seg.replace('✗', &format!("{ANSI_TOOL_CORAL}✗{ANSI_RESET}"));
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

pub(crate) fn ansi_style_cream_verb(verb: &str) -> String {
    format!("{ANSI_BOLD}{ANSI_TOOL_CREAM}{verb}{ANSI_RESET}")
}

pub(crate) fn ansi_style_running_verb(seg: &str) -> String {
    let (colon, body) = tool_line_colon_prefix(seg);
    let verb_end = body.find(' ').unwrap_or(body.len());
    let (verb, tail) = body.split_at(verb_end);
    format!("{colon}{}{}", ansi_style_cream_verb(verb), ansi_style_path_tail(tail))
}

pub(crate) fn ansi_style_done_verb(seg: &str) -> String {
    let (colon, body) = tool_line_colon_prefix(seg);
    for prefix in DONE_VERB_PREFIXES {
        if let Some(tail) = body.strip_prefix(prefix) {
            let verb = prefix.trim_end();
            let mut out = format!("{colon}{}", ansi_style_cream_verb(verb));
            if !tail.is_empty() {
                out.push(' ');
                out.push_str(&ansi_style_path_tail(tail));
            }
            return out;
        }
    }
    format!("{colon}{}", ansi_style_path_tail(body))
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
        || body.starts_with("Search ")
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
    if seg.chars().any(|c| c == '/' || c == '.') || is_byte_size_segment(seg) {
        return format!("{ANSI_DIM}{seg}{ANSI_RESET}");
    }
    format!("{ANSI_TOOL_CREAM}{seg}{ANSI_RESET}")
}

#[cfg(test)]
#[path = "ansi_tests.rs"]
mod ansi_tests;
