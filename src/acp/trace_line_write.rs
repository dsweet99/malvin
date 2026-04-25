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

fn format_trace_display_line(line: &str, kind: Option<SessionUpdateChunkKind>) -> String {
    match kind {
        Some(SessionUpdateChunkKind::Thought) => format!("[{line}]"),
        _ => line.to_string(),
    }
}

fn print_tee_unprefixed_wrapped_line(line: &str) {
    let (max_payload, wrap) = crate::output::terminal_wrap::line_wrap_for_prefix_len(
        0,
        line,
        crate::output::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if !wrap {
        println!("{line}");
        return;
    }
    for seg in crate::output::terminal_wrap::wrap_words_bounded(max_payload, line) {
        println!("{seg}");
    }
}

fn trace_tee_stdout_line(writer: &mut PromptTraceWriter, line: &str, ctx: &TraceTeeStdoutCtx<'_>) {
    if !ctx.tee_stdout {
        return;
    }
    if writer.plain_lines || writer.raw_output {
        print_tee_unprefixed_wrapped_line(line);
        return;
    }
    match writer.stdout_replacement {
        Some(rep) => {
            if !writer.placeholder_emitted {
                crate::output::print_stdout_acp_tee_line_with_timestamp(
                    &crate::output::AcpTeeStdoutEvent {
                        direction: crate::output::AcpTeeDirection::FromAgent,
                        who: &writer.who,
                        line: rep,
                        ts: ctx.ts,
                        emit_stdout_markdown: false,
                    },
                );
                writer.placeholder_emitted = true;
            }
        }
        None => {
            if matches!(ctx.kind, Some(SessionUpdateChunkKind::Thought)) {
                if writer.emit_stdout_markdown {
                    crate::output::print_stdout_acp_tee_line_with_timestamp(
                        &crate::output::AcpTeeStdoutEvent {
                            direction: crate::output::AcpTeeDirection::FromAgent,
                            who: &writer.who,
                            line,
                            ts: ctx.ts,
                            emit_stdout_markdown: true,
                        },
                    );
                } else {
                    crate::output::print_stdout_acp_tee_line_with_timestamp_dim_payload(
                        crate::output::AcpTeeDirection::FromAgent,
                        &writer.who,
                        line,
                        ctx.ts,
                    );
                }
            } else {
                crate::output::print_stdout_acp_tee_line_with_timestamp(
                    &crate::output::AcpTeeStdoutEvent {
                        direction: crate::output::AcpTeeDirection::FromAgent,
                        who: &writer.who,
                        line,
                        ts: ctx.ts,
                        emit_stdout_markdown: writer.emit_stdout_markdown,
                    },
                );
            }
        }
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
    trace_tee_stdout_line(
        writer,
        &display_line,
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

#[test]
fn kiss_stringify_trace_line_write() {
    let _ = stringify!(ReaderTraceLineOpts);
    let _ = stringify!(reader_loop_verbose_and_trace_line);
    let _ = stringify!(TraceTeeStdoutCtx);
    let _ = stringify!(format_trace_display_line);
    let _ = stringify!(print_tee_unprefixed_wrapped_line);
    let _ = stringify!(trace_file_write_line);
    let _ = stringify!(write_trace_line_coalesced);
    let _ = stringify!(WriteTraceLineCoalescedOpts);
    let _ = stringify!(raw_output_should_skip_chunk);
    let _ = stringify!(trace_tee_stdout_line);
}
