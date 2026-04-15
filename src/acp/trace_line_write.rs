//! Trace file line emission and coalesced writes (used by the ACP stdout reader).

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

pub async fn reader_loop_verbose_and_trace_line(
    line: &str,
    opts: &ReaderTraceLineOpts,
    trace_writer: &Arc<Mutex<Option<PromptTraceWriter>>>,
    coalescers: &mut VerboseTraceCoalesceState<'_>,
) {
    let tracing = {
        let g = trace_writer.lock().await;
        g.is_some()
    };
    let parsed: Option<Value> = if opts.acp_verbose || tracing {
        serde_json::from_str(line).ok()
    } else {
        None
    };

    if opts.acp_verbose {
        if let Some((kind, text)) = parsed.as_ref().and_then(session_update_chunk_parts) {
            coalescers.verbose.feed(kind, text.as_str());
        } else {
            coalescers.verbose.flush_all();
            info!(
                target: "malvin::acp::io",
                line = %line,
                "acp message"
            );
        }
    }

    let mut g = trace_writer.lock().await;
    if let Some(ref mut f) = *g {
        write_trace_line_coalesced(f, coalescers.trace, parsed.as_ref(), opts.tee_trace_stdout)
            .await;
    }
}

const fn raw_output_should_skip_chunk(
    kind: Option<SessionUpdateChunkKind>,
    writer: &PromptTraceWriter,
) -> bool {
    writer.raw_output && matches!(kind, Some(SessionUpdateChunkKind::Thought))
}

struct TraceTeeStdoutCtx<'a> {
    tee_stdout: bool,
    kind: Option<SessionUpdateChunkKind>,
    ts: &'a str,
}

fn trace_tee_stdout_line(writer: &mut PromptTraceWriter, line: &str, ctx: &TraceTeeStdoutCtx<'_>) {
    if !ctx.tee_stdout {
        return;
    }
    if raw_output_should_skip_chunk(ctx.kind, writer) {
        return;
    }
    if writer.raw_output {
        let cols = crate::output::terminal_wrap::terminal_columns();
        if crate::output::terminal_wrap::stdout_is_wrappable_terminal()
            && line.chars().count() > cols
        {
            for seg in crate::output::terminal_wrap::wrap_words_bounded(cols, line) {
                println!("{seg}");
            }
        } else {
            println!("{line}");
        }
        return;
    }
    match writer.stdout_replacement {
        Some(rep) => {
            if !writer.placeholder_emitted {
                crate::output::print_stdout_acp_tee_line_with_timestamp(
                    crate::output::AcpTeeDirection::FromAgent,
                    &writer.who,
                    rep,
                    ctx.ts,
                );
                writer.placeholder_emitted = true;
            }
        }
        None => crate::output::print_stdout_acp_tee_line_with_timestamp(
            crate::output::AcpTeeDirection::FromAgent,
            &writer.who,
            line,
            ctx.ts,
        ),
    }
}

pub async fn trace_file_write_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    tee_stdout: bool,
    kind: Option<SessionUpdateChunkKind>,
) {
    if raw_output_should_skip_chunk(kind, writer) {
        return;
    }
    let ts = crate::output::timestamp_now_string();
    let formatted = crate::output::format_line_with_timestamp(&ts, &writer.who, line);
    if let Err(e) = writer.file.write_all(formatted.as_bytes()).await {
        warn!(error = %e, "trace write failed");
        return;
    }
    if let Err(e) = writer.file.write_all(b"\n").await {
        warn!(error = %e, "trace newline failed");
        return;
    }
    trace_tee_stdout_line(
        writer,
        line,
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
    parsed: Option<&Value>,
    tee_stdout: bool,
) {
    if let Some((kind, text)) = parsed.and_then(session_update_chunk_parts) {
        for (kind, tl) in coalesce.feed(kind, text.as_str()) {
            trace_file_write_line(trace_file, &tl, tee_stdout, Some(kind)).await;
        }
        return;
    }
    for (kind, tl) in coalesce.flush_all() {
        trace_file_write_line(trace_file, &tl, tee_stdout, Some(kind)).await;
    }
}

#[test]
fn trace_file_write_line_shares_one_timestamp_for_disk_and_tee() {
    let s = include_str!("trace_line_write.rs");
    let start = s
        .find("pub async fn trace_file_write_line")
        .expect("trace_file_write_line");
    let tail = &s[start..];
    let end = tail[20..].find("\npub async fn ").map_or(tail.len(), |i| i + 20);
    let body = &tail[..end];
    assert!(
        body.contains("let ts = crate::output::timestamp_now_string();")
            && body.contains("format_line_with_timestamp(&ts,")
            && body.contains("trace_tee_stdout_line(")
            && body.contains("&ts"),
        "disk trace and stdout tee must use the same timestamp"
    );
}

#[test]
fn kiss_stringify_trace_line_write() {
    let _ = stringify!(ReaderTraceLineOpts);
    let _ = stringify!(reader_loop_verbose_and_trace_line);
    let _ = stringify!(TraceTeeStdoutCtx);
    let _ = stringify!(trace_file_write_line);
    let _ = stringify!(write_trace_line_coalesced);
}
