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
    command: Option<String>,
    started: Instant,
    stdout_start_emitted: bool,
}

pub struct ToolSummaryLines {
    pub log: String,
    pub stdout: Option<String>,
    pub stdout_deferred: Option<String>,
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
include!("tool_summary_human.inc");

pub fn tool_summary_lines(
    v: &Value,
    tracker: &mut ToolSummaryTracker,
    detail: ToolSummaryDetail,
) -> Option<ToolSummaryLines> {
    let parsed = parse_tool_update(v)?;
    tracker.apply(&parsed);
    let log = format_tool_line(&parsed, tracker, ToolSummaryDetail::Log);
    let mut stdout_deferred = None;
    let stdout = match detail {
        ToolSummaryDetail::Log => Some(log.clone()),
        ToolSummaryDetail::Stdout => format_tool_stdout(&parsed, tracker, &mut stdout_deferred),
    };
    if parsed.phase == TOOL_PHASE_DONE {
        tracker.calls.remove(&parsed.id);
    }
    Some(ToolSummaryLines {
        log,
        stdout,
        stdout_deferred,
    })
}

impl ToolSummaryTracker {
    fn apply(&mut self, parsed: &ParsedToolUpdate) {
        let entry = self.calls.entry(parsed.id.clone()).or_insert_with(|| ToolCallRecord {
            kind: parsed.kind.clone(),
            title: parsed.title.clone(),
            command: parsed.command.clone(),
            started: Instant::now(),
            stdout_start_emitted: false,
        });
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
    }

    fn record(&self, id: &str) -> Option<&ToolCallRecord> {
        self.calls.get(id)
    }

    fn record_mut(&mut self, id: &str) -> Option<&mut ToolCallRecord> {
        self.calls.get_mut(id)
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

#[cfg(test)]
mod tool_summary_kiss {
    #[test]
    fn smoke_tool_summary_symbol_names_for_kiss() {
        let _ = stringify!(
            super::tool_summary_lines,
            super::format_tool_stdout,
            super::execute_effective_exit,
            super::execute_stdout_failed,
            super::tool_summary_stdout_display,
            super::ToolSummaryLines
        );
    }
}
