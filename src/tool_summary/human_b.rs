use serde_json::Value;

use super::format::{edit_paths, start_label};
use super::parse::{LineRange, ParsedToolUpdate};
use super::types::{
    shorten_middle, ToolCallRecord, ToolSummaryTracker,
    TOOL_DISPLAY_MAX_WIDTH, ANSI_BOLD, ANSI_CYAN, ANSI_DIM, ANSI_GREEN, ANSI_RED, ANSI_RESET,
};

pub(crate) fn human_read_subject(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    allow_generic: bool,
) -> Option<String> {
    let rec = tracker.record(&parsed.id);
    if let Some(path) = parsed
        .input_path
        .as_deref()
        .or_else(|| rec.and_then(|r| r.input_path.as_deref()))
        .or_else(|| parsed.raw_output.as_ref().and_then(read_output_path))
    {
        let line_range = parsed
            .input_line_range
            .or_else(|| rec.and_then(|r| r.input_line_range));
        return Some(shorten_subject_path(path, line_range));
    }
    if let Some(label) = read_or_edit_title_label(parsed, rec, "Read") {
        return Some(shorten_middle(&label, TOOL_DISPLAY_MAX_WIDTH));
    }
    allow_generic.then(|| "file".to_string())
}

pub(crate) fn read_output_path(raw: &Value) -> Option<&str> {
    raw.get("path")
        .or_else(|| raw.get("filePath"))
        .and_then(Value::as_str)
}

pub(crate) fn human_edit_subject_path(path: &str) -> String {
    shorten_middle(path, TOOL_DISPLAY_MAX_WIDTH)
}

pub(crate) fn human_edit_subject(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    allow_generic: bool,
) -> Option<String> {
    let rec = tracker.record(&parsed.id);
    if let Some(paths) = parsed.raw_output.as_ref().and_then(edit_paths) {
        if paths.len() == 1 {
            return Some(human_edit_subject_path(&paths[0]));
        }
        return Some(format!("{} files", paths.len()));
    }
    if let Some(path) = parsed
        .input_path
        .as_deref()
        .or_else(|| rec.and_then(|r| r.input_path.as_deref()))
    {
        return Some(human_edit_subject_path(path));
    }
    if let Some(label) = read_or_edit_title_label(parsed, rec, "Edit") {
        return Some(shorten_middle(&label, TOOL_DISPLAY_MAX_WIDTH));
    }
    allow_generic.then(|| "file".to_string())
}

pub(crate) fn human_execute_command(parsed: &ParsedToolUpdate, tracker: &ToolSummaryTracker) -> String {
    let rec = tracker.record(&parsed.id);
    let label = parsed
        .command
        .as_deref()
        .or_else(|| rec.and_then(|r| r.command.as_deref()))
        .unwrap_or_else(|| start_label(parsed, rec));
    let stripped = strip_execute_cd_prefix(label);
    shorten_middle(&escape_tool_subject_fragment(stripped), TOOL_DISPLAY_MAX_WIDTH)
}

pub(crate) fn shorten_subject_path(path: &str, line_range: Option<LineRange>) -> String {
    let suffix = format_line_range_suffix(line_range);
    let suffix_chars = suffix.chars().count();
    let min_path = 8usize;
    let path_width = TOOL_DISPLAY_MAX_WIDTH.saturating_sub(suffix_chars).max(min_path);
    let short_path = shorten_middle(path, path_width);
    format!("{short_path}{suffix}")
}

pub(crate) fn format_line_range_suffix(line_range: Option<LineRange>) -> String {
    let Some(range) = line_range else {
        return String::new();
    };
    range.end.map_or_else(
        || format!(":{}", range.start),
        |end| format!(":{}-{}", range.start, end),
    )
}

pub(crate) fn read_or_edit_title_label(
    parsed: &ParsedToolUpdate,
    rec: Option<&ToolCallRecord>,
    verb: &str,
) -> Option<String> {
    let label = start_label(parsed, rec).trim();
    if label.is_empty() {
        return None;
    }
    let mut stripped = label;
    if let Some(rest) = stripped.strip_prefix(verb) {
        stripped = rest.trim_start();
    }
    if stripped.is_empty() {
        return None;
    }
    if looks_like_path_label(stripped) {
        return Some(stripped.to_string());
    }
    None
}

pub(crate) fn looks_like_path_label(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

pub(crate) fn escape_tool_subject_fragment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

pub(crate) fn strip_execute_cd_prefix(cmd: &str) -> &str {
    let trimmed = cmd.trim();
    let Some(rest) = trimmed.strip_prefix("cd ") else {
        return trimmed;
    };
    if let Some(idx) = rest.find(" && ") {
        return rest[idx + 4..].trim_start();
    }
    if let Some(idx) = rest.find(';') {
        return rest[idx + 1..].trim_start();
    }
    trimmed
}

pub(crate) fn raw_byte_size(raw: &Value) -> Option<usize> {
    raw.get("content")
        .and_then(Value::as_str)
        .map(str::len)
        .or_else(|| raw.get("stdout").and_then(Value::as_str).map(str::len))
}

pub(crate) fn humanize_bytes(n: usize) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{} KB", n.div_ceil(1024))
    } else {
        format!("{} MB", n.div_ceil(1024 * 1024))
    }
}

pub(crate) fn humanize_duration(elapsed: std::time::Duration) -> String {
    let ms = elapsed.as_millis();
    if ms < 1000 {
        return format!("{ms}ms");
    }
    let secs = elapsed.as_secs();
    let tenths = elapsed.subsec_millis() / 100;
    format!("{secs}.{tenths}s")
}

pub fn tool_summary_stdout_display(plain: &str) -> String {
    if !crate::output::stdout_use_color() {
        return plain.to_string();
    }
    apply_tool_summary_ansi(plain)
}

pub(crate) fn apply_tool_summary_ansi(plain: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let mut rest = plain;
    while let Some(idx) = rest.find('·') {
        let (left, right) = rest.split_at(idx);
        out.push_str(&ansi_style_tool_segment(left));
        let _ = write!(out, "{ANSI_DIM}·{ANSI_RESET}");
        rest = right.trim_start_matches('·').trim_start();
    }
    out.push_str(&ansi_style_tool_segment(rest));
    out
}

pub(crate) fn ansi_style_tool_segment(seg: &str) -> String {
    let seg = seg.trim();
    if seg.is_empty() {
        return String::new();
    }
    if seg.contains('✓') {
        return seg.replace('✓', &format!("{ANSI_GREEN}✓{ANSI_RESET}"));
    }
    if seg.contains('✗') {
        return seg.replace('✗', &format!("{ANSI_RED}✗{ANSI_RESET}"));
    }
    ansi_style_tool_segment_running_or_path(seg)
}

pub(crate) fn ansi_style_tool_segment_running_or_path(seg: &str) -> String {
    if seg.ends_with('…') || seg.starts_with("Reading ") || seg.starts_with("Run ") || seg.starts_with("Editing ") || seg == "Searching…"
    {
        let verb_end = seg.find(' ').unwrap_or(seg.len());
        let (verb, tail) = seg.split_at(verb_end);
        return format!(
            "{ANSI_BOLD}{verb}{ANSI_RESET}{}",
            ansi_style_path_tail(tail)
        );
    }
    if seg.starts_with("Read ") || seg.starts_with("Edit ") || seg.starts_with("Search ") {
        let rest = seg.split_once(' ').map_or(seg, |(_, r)| r);
        return format!("{}{}", &seg[..seg.len() - rest.len()], ansi_style_path_tail(rest));
    }
    if seg.contains("matches") || seg.contains("exit ") {
        format!("{ANSI_DIM}{seg}{ANSI_RESET}")
    } else {
        ansi_style_path_tail(seg)
    }
}

pub(crate) fn ansi_style_path_tail(seg: &str) -> String {
    if seg.chars().any(|c| c == '/' || c == '.') {
        return format!("{ANSI_CYAN}{seg}{ANSI_RESET}");
    }
    format!("{ANSI_DIM}{seg}{ANSI_RESET}")
}
