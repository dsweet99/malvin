use crate::deferred_log::types::{DeferredEntry, DeferredPayload};
use crate::output::{
    flush_stdout_acp_tee_line_with_timestamp, flush_stdout_acp_tool_summary_tee,
    flush_stdout_raw_line_with_ts, flush_stdout_rendered_line, AcpTeeDirection, AcpTeeStdoutEvent,
};

pub fn emit_deferred_entry(entry: &DeferredEntry) {
    match &entry.payload {
        DeferredPayload::ToolSummary { plain, display, .. } => {
            let ev = acp_event(entry, plain);
            flush_stdout_acp_tool_summary_tee(&ev, display);
        }
        DeferredPayload::AcpTee {
            line,
            display,
            dim_payload,
        } => {
            let payload = display.as_deref().unwrap_or(line.as_str());
            let ev = AcpTeeStdoutEvent {
                direction: AcpTeeDirection::FromAgent,
                who: &entry.who,
                line: payload,
                ts: &entry.ts,
                emit_stdout_markdown: entry.emit_stdout_markdown,
                dim_payload: *dim_payload,
            };
            flush_stdout_acp_tee_line_with_timestamp(&ev);
        }
        DeferredPayload::RawLine { line } => {
            flush_stdout_raw_line_with_ts(line, Some(&entry.ts));
        }
        DeferredPayload::DisplayLog { display, log } => {
            if crate::output::log_contains_heartbeat(log) {
                let _ = display;
                crate::output::append_stdout_log_line(log);
            } else {
                flush_stdout_rendered_line(display, log);
            }
        }
    }
}

pub(crate) fn acp_event<'a>(entry: &'a DeferredEntry, plain: &'a str) -> AcpTeeStdoutEvent<'a> {
    AcpTeeStdoutEvent {
        direction: AcpTeeDirection::FromAgent,
        who: &entry.who,
        line: plain,
        ts: &entry.ts,
        emit_stdout_markdown: entry.emit_stdout_markdown,
        dim_payload: true,
    }
}

#[cfg(test)]
mod emit_tests {
    use super::acp_event;
    use crate::deferred_log::test_fixtures::test_tool_entry;

    #[test]
    fn acp_event_uses_entry_metadata() {
        let entry = test_tool_entry("Read file · 2ms");
        let ev = acp_event(&entry, "Read file · 2ms");
        assert_eq!(ev.line, "Read file · 2ms");
        assert_eq!(ev.who, entry.who);
        assert_eq!(ev.ts, entry.ts);
    }
}
