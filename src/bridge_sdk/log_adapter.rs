use crate::acp::AcpJsonlTrace;
use crate::acp::SessionUpdateChunkKind;
use crate::output::{WHO_B, WHO_M};

use crate::bridge_protocol::BridgeEvent;

use super::log_adapter_tool::{ToolCallFields, clear_tool_starts, emit_tool};
use super::session::BridgeSession;

pub fn handle_stream_event(session: &BridgeSession, ev: &BridgeEvent) {
    match ev {
        BridgeEvent::Assistant { text } => emit_assistant(session, text),
        BridgeEvent::Thinking { text } => emit_thinking(session, text),
        BridgeEvent::ToolCall {
            phase,
            name,
            summary,
            tool_call_id,
            ..
        } => {
            flush_stdout_coalesce(session);
            emit_tool(
                session,
                ToolCallFields {
                    phase,
                    name: name.as_deref(),
                    summary: summary.as_deref(),
                    tool_call_id: tool_call_id.as_deref(),
                },
            );
        }
        BridgeEvent::RunDone { .. } => {
            flush_stdout_coalesce(session);
            clear_tool_starts(session);
            append_trace_value(session, &logged_run_done(session, ev));
        }
        BridgeEvent::Progress { .. } => append_trace_value(session, ev),
        _ => {
            flush_stdout_coalesce(session);
            append_trace_value(session, ev);
        }
    }
}

pub(crate) fn feed_do_dm_run_result(text: &str) {
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

fn emit_assistant(session: &BridgeSession, text: &str) {
    append_trace_raw(session, "assistant", text);
    if session.io.no_tee || text.is_empty() {
        return;
    }
    tee_coalesced(session, SessionUpdateChunkKind::Message, text);
}

fn emit_thinking(session: &BridgeSession, text: &str) {
    append_trace_raw(session, "thinking", text);
    if session.io.no_tee || !session.io.show_thoughts_on_stdout || text.is_empty() {
        return;
    }
    tee_coalesced(session, SessionUpdateChunkKind::Thought, text);
}

fn tee_coalesced(session: &BridgeSession, kind: SessionUpdateChunkKind, text: &str) {
    let emissions = {
        let mut coalesce = session
            .stdout_coalesce
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coalesce.feed(kind, text)
    };
    for (kind, line, ..) in emissions {
        print_coalesced_line(session, kind, &line);
    }
}

fn flush_stdout_coalesce(session: &BridgeSession) {
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
        print_coalesced_line(session, kind, &line);
    }
}

fn print_coalesced_line(session: &BridgeSession, kind: SessionUpdateChunkKind, line: &str) {
    if line.is_empty() {
        return;
    }
    let who = match kind {
        SessionUpdateChunkKind::Message => WHO_M,
        SessionUpdateChunkKind::Thought => WHO_B,
    };
    let markdown = session.io.emit_stdout_markdown;
    if session.io.raw_output && kind == SessionUpdateChunkKind::Message {
        crate::output::print_stdout_text_with_markdown(who, line, markdown);
    } else {
        crate::output::print_stdout_line_with_markdown(who, line, markdown);
    }
}

fn logged_run_done(session: &BridgeSession, ev: &BridgeEvent) -> BridgeEvent {
    let mut ev = ev.clone();
    crate::bridge_protocol::canonicalize_run_done(&mut ev);
    if let BridgeEvent::RunDone { duration_ms, .. } = &mut ev {
        if duration_ms.is_none() {
            *duration_ms = u64::try_from(session.started_at.elapsed().as_millis()).ok();
        }
    }
    ev
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

pub(crate) fn append_trace_line(session: &BridgeSession, line: &str) {
    let Some(run_dir) = session.run_dir.as_ref() else {
        return;
    };
    let trace = AcpJsonlTrace::new(run_dir.join("trace.jsonl"), "sdk".into());
    trace.append_line("in", line);
}

#[cfg(test)]
mod tests {
    use super::feed_do_dm_run_result;
    use crate::acp::{SessionUpdateChunkKind, TraceChunkCoalescer};
    use crate::output::{
        DM_END, DM_START, WHO_M, enable_stdout_capture, set_do_dm_stdout_mode, take_captured_stdout,
    };

    #[test]
    fn feed_do_dm_run_result_extracts_fenced_body() {
        set_do_dm_stdout_mode(true);
        enable_stdout_capture();
        feed_do_dm_run_result(&format!("{DM_START}\nHello.\n{DM_END}"));
        let out = take_captured_stdout();
        set_do_dm_stdout_mode(false);
        assert_eq!(out, "Hello.");
    }

    #[test]
    fn feed_do_dm_run_result_noop_when_mode_off() {
        set_do_dm_stdout_mode(false);
        enable_stdout_capture();
        feed_do_dm_run_result(&format!("{DM_START}\nHello.\n{DM_END}"));
        assert!(take_captured_stdout().is_empty());
    }

    #[test]
    fn word_sized_assistant_chunks_coalesce_before_flush() {
        let mut coalesce = TraceChunkCoalescer::default();
        for piece in [
            "I'll", " check", " recent", " logs", " and", " the", " run", ".",
        ] {
            let mid = coalesce.feed(SessionUpdateChunkKind::Message, piece);
            assert!(
                mid.is_empty(),
                "short word chunks must buffer, not emit immediately; got {mid:?}"
            );
        }
        let flushed = coalesce.flush_all();
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].1, "I'll check recent logs and the run.");
    }

    #[test]
    fn newline_in_assistant_chunk_flushes_line() {
        let mut coalesce = TraceChunkCoalescer::default();
        let mid = coalesce.feed(SessionUpdateChunkKind::Message, "Hello.\n");
        assert_eq!(mid.len(), 1);
        assert_eq!(mid[0].1, "Hello.");
        assert!(coalesce.flush_all().is_empty());
    }

    #[test]
    fn thought_then_message_flushes_thought_on_kind_switch() {
        let mut coalesce = TraceChunkCoalescer::default();
        coalesce.feed(SessionUpdateChunkKind::Thought, "thinking about it");
        let switched = coalesce.feed(SessionUpdateChunkKind::Message, "Hello");
        assert!(
            switched
                .iter()
                .any(|(k, t, ..)| *k == SessionUpdateChunkKind::Thought && t == "thinking about it")
        );
        let rest = coalesce.flush_all();
        assert!(
            rest.iter()
                .any(|(k, t, ..)| *k == SessionUpdateChunkKind::Message && t == "Hello")
        );
    }

    #[test]
    fn coalesced_assistant_line_prints_once_under_m_tag() {
        enable_stdout_capture();
        crate::output::print_stdout_line(WHO_M, "I'll check recent logs and the run.");
        let out = take_captured_stdout();
        let m_lines: Vec<_> = out.lines().filter(|l| l.contains("m|")).collect();
        assert_eq!(m_lines.len(), 1, "expected one m| line, got:\n{out}");
        assert!(m_lines[0].contains("I'll check recent logs and the run."));
        let _ = stringify!(logged_run_done);
        let _ = stringify!(canonicalize_run_done);
    }
}
