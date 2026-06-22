use serde_json::Value;

use super::format::stderr_headline;
use super::human_a_done::{human_done_line, search_query_from};
use super::human_b::{human_edit_subject, human_execute_command, human_read_subject, humanize_duration};
use super::parse::ParsedToolUpdate;
use super::types::{
    shorten_middle, ToolSummaryTracker, TOOL_PHASE_DONE, TOOL_PHASE_NAMED_STATUS,
    TOOL_PHASE_PENDING, TOOL_PHASE_RUNNING, TOOL_PHASE_START,
};

const TOOL_STDOUT_FAST_COLLAPSE_MS: u128 = 300;
const TOOL_STDOUT_RUNNING_MIN_MS: u128 = 1000;

pub(crate) fn format_tool_stdout(
    parsed: &ParsedToolUpdate,
    tracker: &mut ToolSummaryTracker,
    stdout_deferred: &mut Option<String>,
) -> Option<String> {
    let elapsed = tracker.record(&parsed.id)?.started.elapsed();
    let elapsed_ms = elapsed.as_millis();
    if !tool_stdout_should_emit(parsed, elapsed_ms) {
        return None;
    }
    let line = format_tool_line_human(parsed, tracker, elapsed)?;
    if parsed.phase == TOOL_PHASE_DONE {
        tracker.record_mut(&parsed.id)?.stdout_start_emitted = true;
        return Some(line);
    }
    if parsed.phase == TOOL_PHASE_PENDING {
        return Some(line);
    }
    let rec = tracker.record_mut(&parsed.id)?;
    if !rec.stdout_start_emitted {
        rec.stdout_start_emitted = true;
        *stdout_deferred = Some(line);
        return None;
    }
    Some(line)
}

pub(crate) const fn tool_stdout_should_emit(parsed: &ParsedToolUpdate, elapsed_ms: u128) -> bool {
    match parsed.phase {
        TOOL_PHASE_DONE => true,
        TOOL_PHASE_RUNNING => elapsed_ms >= TOOL_STDOUT_RUNNING_MIN_MS,
        TOOL_PHASE_START => elapsed_ms >= TOOL_STDOUT_FAST_COLLAPSE_MS,
        TOOL_PHASE_PENDING => true,
        TOOL_PHASE_NAMED_STATUS => false,
        _ => false,
    }
}

pub(crate) fn format_tool_line_human(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    elapsed: std::time::Duration,
) -> Option<String> {
    let kind = tracker
        .record(&parsed.id)
        .map_or(parsed.kind.as_str(), |r| r.kind.as_str());
    match parsed.phase {
        TOOL_PHASE_DONE => {
            human_done_line(parsed, tracker, kind, elapsed)
        }
        TOOL_PHASE_RUNNING => human_running_line(parsed, tracker, kind, elapsed),
        TOOL_PHASE_START => human_start_line(parsed, tracker, kind),
        TOOL_PHASE_PENDING => human_start_line(parsed, tracker, kind),
        _ => None,
    }
}

pub(crate) fn human_start_line(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    kind: &str,
) -> Option<String> {
    match kind {
        "read" => human_read_subject(parsed, tracker, false).map(|subject| format!("Reading {subject}…")),
        "search" => Some(super::human_a_done::human_search_start(parsed, tracker)),
        "execute" => {
            let cmd = human_execute_command(parsed, tracker);
            Some(format!("Run {cmd}…"))
        }
        "edit" => human_edit_subject(parsed, tracker, false).map(|subject| format!("Editing {subject}…")),
        _ => None,
    }
}

pub(crate) fn human_running_line(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    kind: &str,
    elapsed: std::time::Duration,
) -> Option<String> {
    let rec = tracker.record(&parsed.id)?;
    if !rec.stdout_start_emitted {
        return human_start_line(parsed, tracker, kind);
    }
    let dur = humanize_duration(elapsed);
    match kind {
        "read" => human_read_subject(parsed, tracker, false)
            .map(|subject| format!("Reading {subject} · {dur}")),
        "search" => search_query_from(parsed, tracker).map_or_else(
            || Some(format!("Searching · {dur}")),
            |q| Some(format!(
                "Searching {} · {dur}",
                shorten_middle(q, super::types::TOOL_DISPLAY_MAX_WIDTH)
            )),
        ),
        "execute" => Some(format!(
            "Run {} · {dur}",
            human_execute_command(parsed, tracker)
        )),
        "edit" => human_edit_subject(parsed, tracker, false)
            .map(|subject| format!("Editing {subject} · {dur}")),
        _ => None,
    }
}

pub(crate) fn execute_effective_exit(parsed: &ParsedToolUpdate, raw: Option<&Value>) -> i64 {
    if let Some(code) = raw.and_then(|r| r.get("exitCode")).and_then(Value::as_i64) {
        return code;
    }
    match parsed.status.as_deref() {
        Some("failed" | "cancelled") => 1,
        _ => 0,
    }
}

pub(crate) fn execute_stdout_failed(parsed: &ParsedToolUpdate, exit: i64, raw: Option<&Value>) -> bool {
    exit != 0
        || matches!(parsed.status.as_deref(), Some("failed" | "cancelled"))
        || raw.and_then(stderr_headline).is_some()
}
