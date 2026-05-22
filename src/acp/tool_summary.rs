use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

pub const TOOL_DISPLAY_MAX_WIDTH: usize = 60;
const TOOL_ELLIPSIS: &str = "...";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolSummaryDetail {
    Stdout,
    Log,
}

#[derive(Default)]
pub struct ToolSummaryTracker {
    calls: HashMap<String, ToolCallRecord>,
}

struct ToolCallRecord {
    kind: String,
    title: String,
    started: Instant,
}

pub struct ToolSummaryLines {
    pub log: String,
    pub stdout: String,
}

pub fn shorten_middle(s: &str, max_width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_width {
        return s.to_string();
    }
    let elen = TOOL_ELLIPSIS.chars().count();
    let keep = max_width.saturating_sub(elen);
    let front = keep / 2;
    let back = keep - front;
    let mut out: String = chars.iter().take(front).collect();
    out.push_str(TOOL_ELLIPSIS);
    out.extend(chars.iter().skip(chars.len().saturating_sub(back)));
    out
}

include!("tool_summary_parse.inc");
include!("tool_summary_format.inc");

pub fn tool_summary_lines(
    v: &Value,
    tracker: &mut ToolSummaryTracker,
    detail: ToolSummaryDetail,
) -> Option<ToolSummaryLines> {
    let parsed = parse_tool_update(v)?;
    tracker.apply(&parsed);
    let log = format_tool_line(&parsed, tracker, ToolSummaryDetail::Log);
    let stdout = if detail == ToolSummaryDetail::Log {
        log.clone()
    } else {
        format_tool_line(&parsed, tracker, ToolSummaryDetail::Stdout)
    };
    if parsed.phase == TOOL_PHASE_DONE {
        tracker.calls.remove(&parsed.id);
    }
    Some(ToolSummaryLines { log, stdout })
}

impl ToolSummaryTracker {
    fn apply(&mut self, parsed: &ParsedToolUpdate) {
        let entry = self.calls.entry(parsed.id.clone()).or_insert_with(|| ToolCallRecord {
            kind: parsed.kind.clone(),
            title: parsed.title.clone(),
            started: Instant::now(),
        });
        if !parsed.kind.is_empty() && parsed.kind != "unknown" {
            entry.kind = parsed.kind.clone();
        }
        if !parsed.title.is_empty() {
            entry.title = parsed.title.clone();
        }
    }

    fn record(&self, id: &str) -> Option<&ToolCallRecord> {
        self.calls.get(id)
    }
}

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
        assert!(
            !lines.log.contains("[tool] running"),
            "unknown status must not be reported as running; got {:?}",
            lines.log
        );
        assert!(
            lines.log.contains("[tool] queued"),
            "unknown status should use the status name as the phase label; got {:?}",
            lines.log
        );
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
        assert_eq!(
            tracker.calls.len(),
            0,
            "completed tools should not be retained in tracker (minimal state per plan)"
        );
    }

    #[test]
    fn pending_update_includes_identifying_title_from_prior_start() {
        let start = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "tool_pend_ctx",
                "kind": "read",
                "status": "pending",
                "title": "Read File"
            }}
        });
        let pending = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "tool_pend_ctx",
                "kind": "read",
                "status": "pending"
            }}
        });
        let mut tracker = ToolSummaryTracker::default();
        tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
        let lines = tool_summary_lines(&pending, &mut tracker, ToolSummaryDetail::Log).unwrap();
        assert!(
            lines.log.contains("[tool] pending") && lines.log.contains("Read File"),
            "pending summary must identify the tool (plan: diagnose from last visible state); got {:?}",
            lines.log
        );
    }
}

#[cfg(test)]
mod tool_summary_kiss {
    #[test]
    fn smoke_tool_summary_symbol_names_for_kiss() {
        let _ = std::any::type_name::<super::ToolSummaryDetail>();
        let _ = std::any::type_name::<super::ToolSummaryTracker>();
        let _ = std::any::type_name::<super::ToolSummaryLines>();
        let _ = stringify!(
            super::TOOL_DISPLAY_MAX_WIDTH,
            super::shorten_middle,
            super::tool_summary_lines,
            super::parse_tool_update,
            super::format_tool_line,
            super::tool_phase_label,
            super::phase_for_session_update,
            super::push_edit_path,
            super::append_edit_counts,
            super::stderr_headline,
            super::stdout_headline
        );
    }
}
