use serde_json::Value;

use super::format::{stderr_headline, stdout_headline};
use super::human_b::{
    human_edit_subject, human_execute_command, human_read_subject, humanize_bytes,
    humanize_duration, raw_byte_size,
};
use super::parse::json_number;
use super::parse::ParsedToolUpdate;
use super::types::{shorten_middle, ToolSummaryTracker, TOOL_DISPLAY_MAX_WIDTH};

pub(crate) fn human_done_line(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    kind: &str,
    elapsed: std::time::Duration,
) -> Option<String> {
    match kind {
        "read" => Some(human_read_done(parsed, tracker, elapsed)),
        "search" => Some(human_search_done(parsed, tracker, elapsed)),
        "execute" => Some(human_execute_done(parsed, tracker, elapsed)),
        "edit" => Some(human_edit_done(parsed, tracker, elapsed)),
        _ => None,
    }
}

pub(crate) fn human_read_done(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    elapsed: std::time::Duration,
) -> String {
    let subject = human_read_subject(parsed, tracker, true).unwrap_or_else(|| "file".to_string());
    let dur = humanize_duration(elapsed);
    let Some(raw) = parsed.raw_output.as_ref() else {
        return format!("Read {subject} · {dur}");
    };
    let size = raw
        .get("content")
        .and_then(Value::as_str)
        .map(|c| humanize_bytes(c.len()))
        .or_else(|| raw_byte_size(raw).map(humanize_bytes));
    size.map_or_else(
        || format!("Read {subject} · {dur}"),
        |size| format!("Read {subject} · {size} · {dur}"),
    )
}

pub(crate) fn human_search_start(parsed: &ParsedToolUpdate, tracker: &ToolSummaryTracker) -> String {
    if let Some(q) = search_query_from(parsed, tracker) {
        return format!("Searching {}…", shorten_middle(q, TOOL_DISPLAY_MAX_WIDTH));
    }
    "Searching…".to_string()
}

pub(crate) fn search_query_from<'a>(
    parsed: &'a ParsedToolUpdate,
    tracker: &'a ToolSummaryTracker,
) -> Option<&'a str> {
    parsed
        .search_query
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            tracker
                .record(&parsed.id)
                .and_then(|r| r.search_query.as_deref())
                .filter(|s| !s.is_empty())
        })
}

fn search_done_line(query: Option<&str>, matches: Option<u64>, truncated: bool) -> String {
    let query_suffix = query
        .map(|q| format!(" {}", shorten_middle(q, TOOL_DISPLAY_MAX_WIDTH)))
        .unwrap_or_default();
    let mut line = matches.map_or_else(
        || format!("Search{query_suffix} · matches"),
        |n| format!("Search{query_suffix} · {n} matches"),
    );
    if truncated {
        line.push_str(" (truncated)");
    }
    line
}

pub(crate) fn human_search_done(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    _elapsed: std::time::Duration,
) -> String {
    let query = search_query_from(parsed, tracker);
    let Some(raw) = parsed.raw_output.as_ref() else {
        return search_done_line(query, None, false);
    };
    let truncated = raw.get("truncated").and_then(Value::as_bool) == Some(true);
    let matches = raw
        .get("totalMatches")
        .or_else(|| raw.get("resultCount"))
        .and_then(json_number);
    search_done_line(query, matches, truncated)
}

pub(crate) fn human_execute_done(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    elapsed: std::time::Duration,
) -> String {
    let cmd = human_execute_command(parsed, tracker);
    let dur = humanize_duration(elapsed);
    let raw = parsed.raw_output.as_ref();
    let exit = super::human_a::execute_effective_exit(parsed, raw);
    if super::human_a::execute_stdout_failed(parsed, exit, raw) {
        let mut line = format!("Run {cmd} · {dur} · ✗ exit {exit}");
        if let Some(r) = raw {
            if let Some(err) = stderr_headline(r).or_else(|| stdout_headline(r)) {
                let short = shorten_middle(err, TOOL_DISPLAY_MAX_WIDTH);
                line.push_str(" · ");
                line.push_str(&short);
            }
        }
        return line;
    }
    format!("Run {cmd} · {dur} · ✓")
}

pub(crate) fn human_edit_done(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    elapsed: std::time::Duration,
) -> String {
    let subject = human_edit_subject(parsed, tracker, true).unwrap_or_else(|| "file".to_string());
    let dur = humanize_duration(elapsed);
    let Some(raw) = parsed.raw_output.as_ref() else {
        return format!("Edit {subject} · {dur}");
    };
    let counts = human_edit_counts(raw);
    if counts.is_empty() {
        return format!("Edit {subject} · {dur}");
    }
    format!("Edit {subject} · {counts} · {dur}")
}

pub(crate) fn human_edit_counts(raw: &Value) -> String {
    let added = raw
        .get("linesAdded")
        .or_else(|| raw.get("added"))
        .and_then(json_number);
    let removed = raw
        .get("linesRemoved")
        .or_else(|| raw.get("removed"))
        .and_then(json_number);
    match (added, removed) {
        (Some(a), Some(r)) => format!("+{a}/−{r}"),
        (Some(a), None) => format!("+{a}"),
        (None, Some(r)) => format!("−{r}"),
        (None, None) => String::new(),
    }
}

#[cfg(test)]
mod search_done_line_tests {
    use super::search_done_line;

    #[test]
    fn search_done_line_covers_query_match_and_truncated_branches() {
        assert_eq!(
            search_done_line(Some("q"), Some(2), false),
            "Search q · 2 matches"
        );
        assert_eq!(search_done_line(None, None, false), "Search · matches");
        assert_eq!(
            search_done_line(Some("q"), None, true),
            "Search q · matches (truncated)"
        );
    }
}
