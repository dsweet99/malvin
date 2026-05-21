// Trace file line emission and coalesced writes (used by the ACP stdout reader).

use super::{
    PromptTraceWriter, SessionUpdateChunkKind, TraceChunkCoalescer, VerboseTraceCoalesceState,
    session_update_chunk_parts,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Flags for [`reader_loop_verbose_and_trace_line`] (avoids multiple `bool` parameters per `kiss`).
#[derive(Clone, Copy)]
pub struct ReaderTraceLineOpts {
    pub acp_verbose: bool,
    pub tee_trace_stdout: bool,
}

pub struct WriteTraceLineCoalescedOpts<'a> {
    pub parsed: Option<&'a Value>,
    pub raw_line: &'a str,
    pub tee_stdout: bool,
}

pub async fn reader_loop_verbose_and_trace_line(
    line: &str,
    opts: &ReaderTraceLineOpts,
    trace_writer: &Arc<Mutex<Option<PromptTraceWriter>>>,
    coalescers: &mut VerboseTraceCoalesceState<'_>,
) {
    let mut g = trace_writer.lock().await;
    let tracing = g.is_some();
    let parsed: Option<Value> = if opts.acp_verbose || tracing {
        serde_json::from_str(line).ok()
    } else {
        None
    };

    if opts.acp_verbose {
        if let Some((kind, text)) = parsed.as_ref().and_then(session_update_chunk_parts) {
            coalescers.verbose.feed(kind, text);
        } else {
            coalescers.verbose.flush_all();
            info!(
                target: "malvin::acp::io",
                line = %line,
                "acp message"
            );
        }
    }

    if let Some(ref mut f) = *g {
        write_trace_line_coalesced(
            f,
            coalescers.trace,
            WriteTraceLineCoalescedOpts {
                parsed: parsed.as_ref(),
                raw_line: line,
                tee_stdout: opts.tee_trace_stdout,
            },
        )
        .await;
    }
}

const fn raw_output_suppress_thought_stdout(
    kind: Option<SessionUpdateChunkKind>,
    writer: &PromptTraceWriter,
) -> bool {
    writer.raw_output
        && matches!(kind, Some(SessionUpdateChunkKind::Thought))
        && !writer.show_thoughts_on_stdout
}

struct TraceTeeStdoutCtx<'a> {
    tee_stdout: bool,
    kind: Option<SessionUpdateChunkKind>,
    ts: &'a str,
}

include!("trace_line_write_tee.inc");

pub async fn trace_file_write_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    tee_stdout: bool,
    kind: Option<SessionUpdateChunkKind>,
) {
    let display_line = format_trace_display_line(line, kind);
    let ts = crate::output::timestamp_now_string();
    let formatted = if writer.plain_lines {
        display_line.clone()
    } else {
        crate::output::format_line_with_timestamp(&ts, &writer.who, &display_line)
    };
    let mut record = formatted.into_bytes();
    record.push(b'\n');
    if let Err(e) = writer.file.write_all(&record).await {
        warn!(error = %e, "trace write failed");
        return;
    }
    if let Err(e) = writer.file.sync_all().await {
        warn!(error = %e, "trace fsync failed");
        return;
    }
    if raw_output_suppress_thought_stdout(kind, writer) {
        return;
    }
    let stdout_line = if writer.plain_lines || writer.raw_output {
        line
    } else {
        &display_line
    };
    trace_tee_stdout_line(
        writer,
        stdout_line,
        &TraceTeeStdoutCtx {
            tee_stdout,
            kind,
            ts: &ts,
        },
    );
}

pub async fn write_trace_line_coalesced(
    trace_file: &mut PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
    opts: WriteTraceLineCoalescedOpts<'_>,
) {
    if let Some((kind, text)) = opts.parsed.and_then(session_update_chunk_parts) {
        for (kind, tl) in coalesce.feed(kind, text) {
            trace_file_write_line(trace_file, &tl, opts.tee_stdout, Some(kind)).await;
        }
        return;
    }
    for (kind, tl) in coalesce.flush_all() {
        trace_file_write_line(trace_file, &tl, opts.tee_stdout, Some(kind)).await;
    }
    let unparsed_tee = opts.tee_stdout && opts.parsed.is_none();
    trace_file_write_line(trace_file, opts.raw_line, unparsed_tee, None).await;
}

#[cfg(test)]
mod trace_line_write_tests {
    use super::*;
    use crate::acp::SessionUpdateChunkKind;

    #[tokio::test]
    async fn trace_line_write_paths_execute_core_and_tee_helpers() {
        assert_eq!(
            format_trace_display_line("x", Some(SessionUpdateChunkKind::Thought)),
            "[x]"
        );
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("t.log");
        let f = tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .await
            .expect("open");
        let mut w = PromptTraceWriter {
            file: f,
            who: "t".into(),
            plain_lines: false,
            stdout_replacement: None,
            placeholder_emitted: false,
            raw_output: false,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
        };
        write_trace_line_coalesced(
            &mut w,
            &mut TraceChunkCoalescer::default(),
            WriteTraceLineCoalescedOpts {
                parsed: None,
                raw_line: "raw",
                tee_stdout: false,
            },
        )
        .await;
        assert!(!raw_output_suppress_thought_stdout(
            Some(SessionUpdateChunkKind::Thought),
            &w
        ));
        let ts = "20260520.000000.000";
        w.stdout_replacement = Some("…");
        trace_tee_stdout_line(
            &mut w,
            "ignored",
            &TraceTeeStdoutCtx {
                tee_stdout: true,
                kind: Some(SessionUpdateChunkKind::Message),
                ts,
            },
        );
        assert!(w.placeholder_emitted);
        w.stdout_replacement = None;
        w.placeholder_emitted = false;
        let event = trace_tee_stdout_event(&w, "thought", ts, true);
        assert_eq!(
            event.direction,
            crate::output::AcpTeeDirection::FromAgent
        );
        trace_tee_stdout_line(
            &mut w,
            "thought",
            &TraceTeeStdoutCtx {
                tee_stdout: true,
                kind: Some(SessionUpdateChunkKind::Thought),
                ts,
            },
        );
        w.raw_output = true;
        assert!(raw_output_suppress_thought_stdout(
            Some(SessionUpdateChunkKind::Thought),
            &w
        ));
        print_tee_unprefixed_wrapped_line("wrap-me");
        trace_file_write_line(&mut w, "tee", true, Some(SessionUpdateChunkKind::Message)).await;
        let arc = Arc::new(Mutex::new(Some(w)));
        let mut verbose = crate::acp::VerboseIoCoalescer::default();
        let mut trace_c = TraceChunkCoalescer::default();
        reader_loop_verbose_and_trace_line(
            r#"{"jsonrpc":"2.0"}"#,
            &ReaderTraceLineOpts {
                acp_verbose: true,
                tee_trace_stdout: false,
            },
            &arc,
            &mut VerboseTraceCoalesceState {
                verbose: &mut verbose,
                trace: &mut trace_c,
            },
        )
        .await;
        drop(arc);
        let body = tokio::fs::read_to_string(&path).await.expect("read");
        assert!(body.contains("raw") && body.contains("tee"));
    }
}
