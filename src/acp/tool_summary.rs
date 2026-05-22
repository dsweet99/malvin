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
