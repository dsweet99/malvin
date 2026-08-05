//! Stdout / trace adapter for bridge events (VISION log classes).

use crate::acp::AcpJsonlTrace;
use crate::output::{AcpTeeDirection, AcpTeeStdoutEvent, WHO_B, WHO_M, WHO_T};
use crate::tool_summary::tool_summary_stdout_display;

use super::protocol::BridgeEvent;
use super::session::BridgeSession;

pub fn handle_stream_event(session: &BridgeSession, ev: &BridgeEvent) {
    match ev {
        BridgeEvent::Assistant { text } => emit_assistant(session, text),
        BridgeEvent::Thinking { text } => emit_thinking(session, text),
        BridgeEvent::ToolCall {
            phase,
            name,
            summary,
            ..
        } => emit_tool(session, phase, name.as_deref(), summary.as_deref()),
        _ => append_trace_value(session, ev),
    }
}

fn emit_assistant(session: &BridgeSession, text: &str) {
    append_trace_raw(session, "assistant", text);
    if session.io.no_tee || text.is_empty() {
        return;
    }
    for chunk in non_empty_lines(text) {
        if session.io.raw_output {
            crate::output::print_stdout_text(WHO_M, chunk);
        } else {
            crate::output::print_stdout_line(WHO_M, chunk);
        }
    }
}

fn emit_thinking(session: &BridgeSession, text: &str) {
    append_trace_raw(session, "thinking", text);
    if session.io.no_tee || !session.io.show_thoughts_on_stdout || text.is_empty() {
        return;
    }
    for chunk in non_empty_lines(text) {
        crate::output::print_stdout_line(WHO_B, chunk);
    }
}

fn emit_tool(session: &BridgeSession, phase: &str, name: Option<&str>, summary: Option<&str>) {
    let payload = serde_json::json!({
        "event": "tool_call",
        "phase": phase,
        "name": name,
        "summary": summary,
    });
    append_trace_line(session, &payload.to_string());
    if session.io.no_tee || session.io.raw_output || phase != "start" {
        return;
    }
    let plain = summary.or(name).unwrap_or("tool");
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

fn append_trace_value(session: &BridgeSession, ev: &BridgeEvent) {
    if let Ok(raw) = serde_json::to_string(ev) {
        append_trace_line(session, &raw);
    }
}

fn append_trace_raw(session: &BridgeSession, kind: &str, text: &str) {
    let raw = serde_json::json!({ "event": kind, "text": text }).to_string();
    append_trace_line(session, &raw);
}

fn append_trace_line(session: &BridgeSession, line: &str) {
    let Some(run_dir) = session.run_dir.as_ref() else {
        return;
    };
    let trace = AcpJsonlTrace::new(run_dir.join("trace.jsonl"), "sdk".into());
    trace.append_line("in", line);
}

fn non_empty_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().filter(|l| !l.is_empty())
}
