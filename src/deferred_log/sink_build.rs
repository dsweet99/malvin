use std::time::Instant;

use super::types::{AcpTeeBuild, DeferredEntry, DeferredPayload, ToolSummaryBuild};

pub fn build_tool_entry(build: ToolSummaryBuild) -> DeferredEntry {
    DeferredEntry {
        enqueued_at: Instant::now(),
        who: build.tee.who,
        ts: build.tee.ts,
        emit_stdout_markdown: build.tee.emit_stdout_markdown,
        kind: None,
        payload: DeferredPayload::ToolSummary {
            plain: build.plain,
            display: build.display,
            enrich: build.enrich,
            meta: build.meta,
        },
    }
}

pub fn build_acp_tee_entry(build: AcpTeeBuild) -> DeferredEntry {
    DeferredEntry {
        enqueued_at: Instant::now(),
        who: build.tee.who,
        ts: build.tee.ts,
        emit_stdout_markdown: build.tee.emit_stdout_markdown,
        kind: build.kind,
        payload: DeferredPayload::AcpTee {
            line: build.line,
            display: build.display,
            dim_payload: build.dim_payload,
        },
    }
}

pub fn build_raw_line_entry(line: String, who: String, ts: String) -> DeferredEntry {
    DeferredEntry {
        enqueued_at: Instant::now(),
        who,
        ts,
        emit_stdout_markdown: false,
        kind: None,
        payload: DeferredPayload::RawLine { line },
    }
}

pub fn build_display_log_entry(display: String, log: String) -> DeferredEntry {
    DeferredEntry {
        enqueued_at: Instant::now(),
        who: String::new(),
        ts: String::new(),
        emit_stdout_markdown: false,
        kind: None,
        payload: DeferredPayload::DisplayLog { display, log },
    }
}
