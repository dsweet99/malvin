use std::time::{Duration, Instant};

use crate::output::{AcpTeeDirection, AcpTeeStdoutEvent, WHO_T};
use crate::tool_summary::{humanize_duration, tool_summary_stdout_display};

use super::log_adapter::append_trace_line;
use super::session::ToolCallStart;
use super::stream_log::StreamLog;

pub(crate) struct ToolCallFields<'a> {
    pub phase: &'a str,
    pub name: Option<&'a str>,
    pub summary: Option<&'a str>,
    pub tool_call_id: Option<&'a str>,
}

pub(crate) fn emit_tool(session: &StreamLog, fields: ToolCallFields<'_>) {
    let ToolCallFields {
        phase,
        name,
        summary,
        tool_call_id,
    } = fields;
    let payload = serde_json::json!({
        "event": "tool_call",
        "phase": phase,
        "name": name,
        "summary": summary,
        "toolCallId": tool_call_id,
    });
    append_trace_line(session, &payload.to_string());
    if session.io.no_tee || session.io.raw_output {
        return;
    }
    let subject = summary.or(name).unwrap_or("tool");
    let phase = match phase {
        "end" | "done" => "complete",
        other => other,
    };
    match phase {
        "start" => note_tool_start(session, tool_call_id, subject),
        "complete" | "error" => {
            let plain = format_tool_done_line(
                session,
                &DoneLineInput {
                    tool_call_id,
                    subject,
                    name,
                    phase,
                },
            );
            tee_tool_line(session, &plain);
        }
        _ => {}
    }
}

pub(crate) fn clear_tool_starts(session: &StreamLog) {
    session
        .tool_starts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}

const ANON_TOOL_ID: &str = "__malvin_anon_tool__";

/// Record a tool start; empty `toolCallId` uses [`ANON_TOOL_ID`] so `tools_in_flight` still arms.
fn note_tool_start(session: &StreamLog, tool_call_id: Option<&str>, summary: &str) {
    let id = tool_call_id
        .filter(|s| !s.is_empty())
        .unwrap_or(ANON_TOOL_ID);
    let mut starts = session
        .tool_starts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    starts.insert(
        id.to_string(),
        ToolCallStart {
            started: Instant::now(),
            summary: summary.to_string(),
        },
    );
}

fn take_tool_start(session: &StreamLog, tool_call_id: Option<&str>) -> Option<ToolCallStart> {
    let mut starts = session
        .tool_starts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(id) = tool_call_id.filter(|s| !s.is_empty()) {
        return starts.remove(id);
    }
    starts.remove(ANON_TOOL_ID)
}

struct DoneLineInput<'a> {
    tool_call_id: Option<&'a str>,
    subject: &'a str,
    name: Option<&'a str>,
    phase: &'a str,
}

fn format_tool_done_line(session: &StreamLog, input: &DoneLineInput<'_>) -> String {
    let start = take_tool_start(session, input.tool_call_id);
    let from_start = start.as_ref().map_or("", |s| s.summary.as_str());
    let base = if input.subject.len() >= from_start.len() {
        input.subject
    } else if !from_start.is_empty() {
        from_start
    } else {
        input.subject
    };
    let elapsed = start
        .as_ref()
        .map_or(Duration::ZERO, |s| s.started.elapsed());
    compose_tool_done_line(base, input.name, input.phase, elapsed)
}

pub(super) fn compose_tool_done_line(
    base: &str,
    name: Option<&str>,
    phase: &str,
    elapsed: Duration,
) -> String {
    let dur = humanize_duration(elapsed);
    let is_run = base.starts_with("Run ")
        || name.is_some_and(|n| n.eq_ignore_ascii_case("shell") || n.eq_ignore_ascii_case("bash"));
    if phase == "error" {
        return format!("{base} · {dur} · ✗");
    }
    if is_run {
        format!("{base} · {dur} · ✓")
    } else {
        format!("{base} · {dur}")
    }
}

fn tee_tool_line(session: &StreamLog, plain: &str) {
    let display = tool_summary_stdout_display(plain);
    let ts = crate::output::timestamp_now_string();
    crate::output::print_stdout_acp_tool_summary_tee(
        &AcpTeeStdoutEvent {
            direction: AcpTeeDirection::FromAgent,
            who: WHO_T,
            line: plain,
            ts: &ts,
            emit_stdout_markdown: session.io.emit_stdout_markdown,
            dim_payload: false,
        },
        &display,
    );
}

#[cfg(test)]
mod tests {
    use super::compose_tool_done_line;
    use std::time::Duration;

    #[test]
    fn compose_tool_done_line_run_success() {
        let line = compose_tool_done_line(
            "Run ls -ltr",
            Some("shell"),
            "complete",
            Duration::from_millis(12),
        );
        assert_eq!(line, "Run ls -ltr · 12ms · ✓");
    }

    #[test]
    fn compose_tool_done_line_run_error() {
        let line = compose_tool_done_line(
            "Run false · exit 1",
            Some("shell"),
            "error",
            Duration::from_millis(5),
        );
        assert_eq!(line, "Run false · exit 1 · 5ms · ✗");
    }

    #[test]
    fn compose_tool_done_line_read_with_size() {
        let line = compose_tool_done_line(
            "Read README.md · 183 B",
            Some("read"),
            "complete",
            Duration::from_millis(200),
        );
        assert_eq!(line, "Read README.md · 183 B · 200ms");
    }
}
