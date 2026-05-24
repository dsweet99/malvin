use std::fmt::Write as _;

use super::types::{
    ANSI_BOLD, ANSI_RESET, ANSI_TOOL_CORAL, ANSI_TOOL_CREAM, ANSI_TOOL_SAND, ANSI_TOOL_TEAL,
};

const DONE_VERB_PREFIXES: &[&str] = &["Read ", "Edit ", "Search ", "Run "];

pub fn tool_summary_stdout_display(plain: &str) -> String {
    if !crate::output::stdout_use_color() {
        return plain.to_string();
    }
    apply_tool_summary_ansi(plain)
}

fn split_outer_brackets(plain: &str) -> (&str, &str, &str) {
    plain
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .map_or(("", plain, ""), |inner| ("[", inner, "]"))
}

pub(crate) fn apply_tool_summary_ansi(plain: &str) -> String {
    let (open, inner, close) = split_outer_brackets(plain);
    let mut out = String::from(open);
    let mut rest = inner;
    while let Some(idx) = rest.find('·') {
        let (left, right) = rest.split_at(idx);
        out.push_str(&ansi_style_tool_segment(left));
        let _ = write!(out, "{ANSI_TOOL_CREAM}·{ANSI_RESET}");
        rest = right.trim_start_matches('·').trim_start();
    }
    out.push_str(&ansi_style_tool_segment(rest));
    out.push_str(close);
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

fn tool_line_colon_prefix(seg: &str) -> (&str, &str) {
    if let Some(rest) = seg.strip_prefix(":: ") {
        return (":: ", rest);
    }
    if let Some(inner) = seg.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        return ("[", inner);
    }
    ("", seg)
}

fn ansi_style_cream_verb(verb: &str) -> String {
    format!("{ANSI_BOLD}{ANSI_TOOL_CREAM}{verb}{ANSI_RESET}")
}

fn ansi_style_running_verb(seg: &str) -> String {
    let (colon, body) = tool_line_colon_prefix(seg);
    let verb_end = body.find(' ').unwrap_or(body.len());
    let (verb, tail) = body.split_at(verb_end);
    format!("{colon}{}{}", ansi_style_cream_verb(verb), ansi_style_path_tail(tail))
}

fn ansi_style_done_verb(seg: &str) -> String {
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
        format!("{ANSI_TOOL_SAND}{seg}{ANSI_RESET}")
    } else {
        ansi_style_path_tail(seg)
    }
}

fn is_byte_size_segment(seg: &str) -> bool {
    seg.ends_with(" B") || seg.ends_with(" KB") || seg.ends_with(" MB")
}

pub(crate) fn ansi_style_path_tail(seg: &str) -> String {
    if seg.chars().any(|c| c == '/' || c == '.') || is_byte_size_segment(seg) {
        return format!("{ANSI_TOOL_TEAL}{seg}{ANSI_RESET}");
    }
    format!("{ANSI_TOOL_CREAM}{seg}{ANSI_RESET}")
}

#[cfg(test)]
mod ansi_tests {
    use super::{
        ansi_style_cream_verb, ansi_style_done_verb, ansi_style_running_verb,
        apply_tool_summary_ansi, tool_line_colon_prefix,
    };
    use crate::tool_summary::types::{ANSI_BOLD, ANSI_RESET, ANSI_TOOL_CREAM, ANSI_TOOL_TEAL};

    #[test]
    fn covers_running_and_done_helpers() {
        assert!(ansi_style_running_verb("Reading path…").contains("Reading"));
        assert!(ansi_style_done_verb("Read path · 1ms").contains("Read"));
    }

    #[test]
    fn tool_line_colon_prefix_splits_leading_marker() {
        assert_eq!(tool_line_colon_prefix(":: Run x"), (":: ", "Run x"));
        assert_eq!(tool_line_colon_prefix("[Run x]"), ("[", "Run x"));
        assert_eq!(tool_line_colon_prefix("Run x"), ("", "Run x"));
    }

    #[test]
    fn ansi_style_cream_verb_wraps_verb_in_palette() {
        let styled = ansi_style_cream_verb("Edit");
        assert!(styled.contains("Edit"));
        assert!(styled.contains(ANSI_TOOL_CREAM));
    }

    #[test]
    fn bracket_wrapped_running_line_bolds_run_verb() {
        let styled = apply_tool_summary_ansi("[Run echo hi…]");
        let run_verb = format!("{ANSI_BOLD}{ANSI_TOOL_CREAM}Run");
        assert!(
            styled.contains(&run_verb),
            "expected cream bold on Run inside brackets; got {styled:?}"
        );
    }

    #[test]
    fn bracket_wrapped_done_line_bolds_run_verb() {
        let styled = apply_tool_summary_ansi("[Run echo hi · 1ms · ✓]");
        let run_verb = format!("{ANSI_BOLD}{ANSI_TOOL_CREAM}Run");
        assert!(
            styled.contains(&run_verb),
            "expected cream bold on Run in done line; got {styled:?}"
        );
        assert!(styled.contains('['));
    }

    #[test]
    fn bracket_wrapped_reading_running_line_bolds_verb() {
        let styled = apply_tool_summary_ansi("[Reading ./src/foo.rs…]");
        let verb = format!("{ANSI_BOLD}{ANSI_TOOL_CREAM}Reading");
        assert!(
            styled.contains(&verb),
            "expected cream bold on Reading; got {styled:?}"
        );
    }

    #[test]
    fn done_line_bolds_read_verb_without_colon_prefix() {
        let styled = apply_tool_summary_ansi("Read ./src/foo.rs · 1ms");
        let verb = format!("{ANSI_BOLD}{ANSI_TOOL_CREAM}Read");
        assert!(
            styled.contains(&verb),
            "expected cream bold on Read; got {styled:?}"
        );
    }

    #[test]
    fn byte_size_suffixes_use_teal() {
        for (plain, segment) in [
            ("Read file.bbb · 123 B · 1ms", "123 B"),
            ("Read x · 4 KB · 1ms", "4 KB"),
            ("Read x · 2 MB · 1ms", "2 MB"),
        ] {
            let styled = apply_tool_summary_ansi(plain);
            let teal = format!("{ANSI_TOOL_TEAL}{segment}{ANSI_RESET}");
            assert!(styled.contains(&teal), "got {styled:?}");
        }
    }

    #[test]
    fn split_outer_brackets_and_byte_size_segments() {
        use super::{is_byte_size_segment, split_outer_brackets};

        assert_eq!(split_outer_brackets("[Read x]"), ("[", "Read x", "]"));
        assert_eq!(split_outer_brackets("plain"), ("", "plain", ""));
        assert!(is_byte_size_segment("123 B"));
        assert!(!is_byte_size_segment("foo"));
        let styled = apply_tool_summary_ansi("[Read x · 4 KB]");
        assert!(styled.starts_with('[') && styled.contains(']'));
    }

    #[test]
    fn edit_search_and_editing_verbs_use_bold_cream() {
        let cream = format!("{ANSI_BOLD}{ANSI_TOOL_CREAM}");
        for plain in [
            "Edit src/foo.rs · 1ms",
            "Editing src/foo.rs…",
            "Searching rg foo…",
            "Search rg needle · 1ms",
        ] {
            assert!(
                apply_tool_summary_ansi(plain).contains(&cream),
                "got {plain:?}"
            );
        }
    }

    #[test]
    fn styled_running_and_done_lines_use_palette() {
        let running = apply_tool_summary_ansi("Reading ./src/foo.rs…");
        let done = apply_tool_summary_ansi("Read ./src/foo.rs · 1ms");
        assert!(running.contains("Reading"));
        assert!(done.contains("Read"));
    }
}
