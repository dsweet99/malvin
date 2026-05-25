use crate::deferred_log::types::{DeferredEntry, DeferredPayload};
use crate::output::{
    print_stdout_acp_tee_line_with_timestamp, print_stdout_acp_tool_summary_tee,
    print_stdout_raw_line_with_ts, AcpTeeDirection, AcpTeeStdoutEvent,
};

pub fn emit_deferred_entry(entry: &DeferredEntry) {
    match &entry.payload {
        DeferredPayload::ToolSummary { plain, display, .. } => {
            let ev = acp_event(entry, plain);
            print_stdout_acp_tool_summary_tee(&ev, display);
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
            print_stdout_acp_tee_line_with_timestamp(&ev);
        }
        DeferredPayload::RawLine { line } => {
            print_stdout_raw_line_with_ts(line, Some(&entry.ts));
        }
        DeferredPayload::Heartbeat { log_line } => {
            println!("{log_line}");
            crate::output::append_stdout_log_line(log_line);
        }
        DeferredPayload::TaggedStdout { display, log } => {
            println!("{display}");
            crate::output::append_stdout_log_line(log);
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
