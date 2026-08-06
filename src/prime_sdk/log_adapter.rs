//! Stdout / trace adapter for bridge events (VISION log classes).

use crate::acp::SessionUpdateChunkKind;
use crate::acp::AcpJsonlTrace;
use crate::output::{WHO_B, WHO_M};

use super::log_adapter_tool::{prime_clear_tool_starts, prime_emit_tool, PrimeToolCallFields};
use super::protocol::PrimeBridgeEvent;
use super::session::PrimeBridgeSession;

pub fn prime_handle_stream_event(session: &PrimeBridgeSession, ev: &PrimeBridgeEvent) {
    match ev {
        PrimeBridgeEvent::Assistant { text } => prime_emit_assistant(session, text),
        PrimeBridgeEvent::Thinking { text } => prime_emit_thinking(session, text),
        PrimeBridgeEvent::ToolCall {
            phase,
            name,
            summary,
            tool_call_id,
            ..
        } => {
            prime_flush_stdout_coalesce(session);
            prime_emit_tool(
                session,
                PrimeToolCallFields {
                    phase,
                    name: name.as_deref(),
                    summary: summary.as_deref(),
                    tool_call_id: tool_call_id.as_deref(),
                },
            );
        }
        PrimeBridgeEvent::RunDone { .. } => {
            prime_flush_stdout_coalesce(session);
            prime_clear_tool_starts(session);
            prime_append_trace_value(session, ev);
        }
        _ => {
            prime_flush_stdout_coalesce(session);
            prime_append_trace_value(session, ev);
        }
    }
}

/// Feed `run_done.result` into the `--do` / `--quiet` DM extractor.
///
/// The Prime SDK often puts `MALVIN_DM_*` fences only on the final result string while
/// streamed `assistant` events omit them. ACP tees message chunks into the same filter;
/// without this, DM-only stdout stays empty even when the agent answered correctly.
pub(super) fn prime_feed_do_dm_run_result(text: &str) {
    if !crate::output::do_dm_stdout_mode() || text.is_empty() {
        return;
    }
    let mut terminated = String::with_capacity(text.len() + 1);
    terminated.push_str(text);
    if !terminated.ends_with('\n') {
        terminated.push('\n');
    }
    crate::output::feed_do_dm_stdout_text(&terminated);
}

fn prime_emit_assistant(session: &PrimeBridgeSession, text: &str) {
    prime_append_trace_raw(session, "assistant", text);
    if session.io.no_tee || text.is_empty() {
        return;
    }
    prime_tee_coalesced(session, SessionUpdateChunkKind::Message, text);
}

fn prime_emit_thinking(session: &PrimeBridgeSession, text: &str) {
    prime_append_trace_raw(session, "thinking", text);
    if session.io.no_tee || !session.io.show_thoughts_on_stdout || text.is_empty() {
        return;
    }
    prime_tee_coalesced(session, SessionUpdateChunkKind::Thought, text);
}

fn prime_tee_coalesced(session: &PrimeBridgeSession, kind: SessionUpdateChunkKind, text: &str) {
    let emissions = {
        let mut coalesce = session
            .stdout_coalesce
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coalesce.feed(kind, text)
    };
    for (kind, line, ..) in emissions {
        prime_print_coalesced_line(session, kind, &line);
    }
}

fn prime_flush_stdout_coalesce(session: &PrimeBridgeSession) {
    if session.io.no_tee {
        let _ = session
            .stdout_coalesce
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .flush_all();
        return;
    }
    let emissions = {
        let mut coalesce = session
            .stdout_coalesce
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coalesce.flush_all()
    };
    for (kind, line, ..) in emissions {
        if kind == SessionUpdateChunkKind::Thought && !session.io.show_thoughts_on_stdout {
            continue;
        }
        prime_print_coalesced_line(session, kind, &line);
    }
}

fn prime_print_coalesced_line(session: &PrimeBridgeSession, kind: SessionUpdateChunkKind, line: &str) {
    if line.is_empty() {
        return;
    }
    let who = match kind {
        SessionUpdateChunkKind::Message => WHO_M,
        SessionUpdateChunkKind::Thought => WHO_B,
    };
    // Prime always uses line-oriented tee for thoughts (never raw thought blobs).
    if session.io.raw_output && kind == SessionUpdateChunkKind::Message {
        crate::output::print_stdout_text(who, line);
        return;
    }
    crate::output::print_stdout_line(who, line);
}

fn prime_append_trace_value(session: &PrimeBridgeSession, ev: &PrimeBridgeEvent) {
    if let Ok(raw) = serde_json::to_string(ev) {
        prime_append_trace_line(session, &raw);
    }
}

fn prime_append_trace_raw(session: &PrimeBridgeSession, kind: &str, text: &str) {
    let raw = serde_json::json!({ "event": kind, "text": text }).to_string();
    prime_append_trace_line(session, &raw);
}

pub(super) fn prime_append_trace_line(session: &PrimeBridgeSession, line: &str) {
    let Some(run_dir) = session.run_dir.as_ref() else {
        return;
    };
    let trace = AcpJsonlTrace::new(run_dir.join("trace.jsonl"), "sdk".into());
    trace.append_line("in", line);
}
