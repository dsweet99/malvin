mod classify_bash;
mod types;
mod parse;
mod parse_acp;
mod format;
mod human_b;
mod ansi;
mod human_a_done;
mod human_a;
#[cfg(test)]
mod search_coverage;
#[cfg(test)]
mod kiss_coverage;
#[cfg(test)]
mod smoke_coverage;

pub use classify_bash::{
    bash_kind_wire_name, classify_bash_command, format_classified_tool_line,
    tool_comment_log_prefix, BashToolKind, ClassifiedToolLineInput,
    TOOL_COMMENT_LOG_PREFIX_CHARS,
};
#[allow(unused_imports)]
pub use types::{
    ToolSummaryDetail, ToolSummaryLines, ToolSummaryTracker, TOOL_DISPLAY_MAX_WIDTH,
    shorten_middle,
};
#[allow(unused_imports)]
pub(crate) use human_a::{execute_effective_exit, execute_stdout_failed};
pub use ansi::tool_summary_stdout_display;
#[cfg(test)]
pub(crate) use ansi::apply_tool_summary_ansi;
#[allow(unused_imports)]
pub(crate) use human_b::relativize_tool_path;
pub(crate) use human_b::humanize_duration;
pub(crate) use human_b::escape_tool_subject_fragment;
#[allow(unused_imports)]
pub(crate) use parse::{json_number, parse_tool_update, tool_phase_label, LineRange, ParsedToolUpdate};
#[allow(unused_imports)]
pub(crate) use parse_acp::acp_line_range_field;
#[allow(unused_imports)]
pub(crate) use human_a_done::{human_done_line, human_read_done};
#[allow(unused_imports)]
pub(crate) use types::{TOOL_PHASE_DONE, TOOL_PHASE_RUNNING, TOOL_PHASE_START};

use types::ToolCallRecord;
use format::format_tool_line;
use human_a::format_tool_stdout;

pub fn tool_summary_lines(
    v: &serde_json::Value,
    tracker: &mut ToolSummaryTracker,
    detail: ToolSummaryDetail,
) -> Option<ToolSummaryLines> {
    let parsed = parse_tool_update(v)?;
    tracker.apply(&parsed);
    crate::agent_phase::observe_tool_update(&parsed, tracker);
    let log = format_tool_line(&parsed, tracker, ToolSummaryDetail::Log);
    let mut stdout_deferred = None;
    let stdout = match detail {
        ToolSummaryDetail::Log => Some(log.clone()),
        ToolSummaryDetail::Stdout => format_tool_stdout(&parsed, tracker, &mut stdout_deferred),
    };
    if parsed.phase == TOOL_PHASE_DONE {
        tracker.record_tool_done(&parsed.id);
        if !matches!(detail, ToolSummaryDetail::Stdout) {
            tracker.calls.remove(&parsed.id);
        }
    }
    Some(ToolSummaryLines {
        log,
        stdout,
        stdout_deferred,
    })
}

fn new_tool_call_record(parsed: &ParsedToolUpdate) -> ToolCallRecord {
    ToolCallRecord {
        kind: parsed.kind.clone(),
        title: parsed.title.clone(),
        command: parsed.command.clone(),
        input_path: parsed.input_path.clone(),
        search_query: parsed.search_query.clone(),
        input_line_range: parsed.input_line_range,
        started: std::time::Instant::now(),
        stdout_start_emitted: false,
    }
}

fn merge_parsed_into_record(entry: &mut ToolCallRecord, parsed: &ParsedToolUpdate) {
    if !parsed.kind.is_empty() && parsed.kind != "unknown" {
        entry.kind = parsed.kind.clone();
    }
    if !parsed.title.is_empty() {
        entry.title = parsed.title.clone();
    }
    if let Some(cmd) = parsed.command.as_ref() {
        entry.command = Some(cmd.clone());
        if entry.title.is_empty() {
            entry.title = cmd.clone();
        }
    }
    if let Some(path) = parsed.input_path.as_ref() {
        entry.input_path = Some(path.clone());
    }
    if let Some(query) = parsed.search_query.as_ref() {
        entry.search_query = Some(query.clone());
    }
    if let Some(line_range) = parsed.input_line_range {
        entry.input_line_range = Some(line_range);
    }
}

impl ToolSummaryTracker {
    fn apply(&mut self, parsed: &ParsedToolUpdate) {
        if let Some(entry) = self.calls.get_mut(&parsed.id) {
            merge_parsed_into_record(entry, parsed);
        } else {
            self.calls
                .insert(parsed.id.clone(), new_tool_call_record(parsed));
        }
    }

    #[cfg(test)]
    pub(crate) fn stored_command<'a>(&'a self, id: &str) -> Option<&'a str> {
        self.calls.get(id).and_then(|r| r.command.as_deref())
    }

    #[cfg(test)]
    pub(crate) fn call_count(&self) -> usize {
        self.calls.len()
    }
}

#[cfg(test)]
mod run_timing_regressions;

#[cfg(test)]
mod tool_summary_regressions {
    use super::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
    use serde_json::json;

    #[test]
    fn tool_call_update_unknown_status_must_not_be_labeled_running() {
        let v = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "tool_unknown_status",
                "kind": "execute",
                "status": "queued"
            }}
        });
        let mut tracker = ToolSummaryTracker::default();
        let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
        assert!(!lines.log.contains("[tool] running"));
        assert!(lines.log.contains("[tool] queued"));
    }

    #[test]
    fn tracker_stores_command_from_start_through_done() {
        let start = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "tool_cmd",
                "kind": "execute",
                "status": "pending",
                "rawInput": {"command": "echo hi"}
            }}
        });
        let running = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "tool_cmd",
                "status": "in_progress"
            }}
        });
        let mut tracker = ToolSummaryTracker::default();
        tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
        assert_eq!(tracker.stored_command("tool_cmd"), Some("echo hi"));
        tool_summary_lines(&running, &mut tracker, ToolSummaryDetail::Log).unwrap();
        assert_eq!(tracker.stored_command("tool_cmd"), Some("echo hi"));
    }

    #[test]
    fn tracker_drops_record_after_tool_call_completes() {
        let start = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "tool_evict",
                "kind": "read",
                "status": "pending",
                "title": "Read"
            }}
        });
        let done = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "tool_evict",
                "kind": "read",
                "status": "completed",
                "rawOutput": {"content": "x"}
            }}
        });
        let mut tracker = ToolSummaryTracker::default();
        tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
        tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Log).unwrap();
        assert_eq!(tracker.call_count(), 0);
    }
}

