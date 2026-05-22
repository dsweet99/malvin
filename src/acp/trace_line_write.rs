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

#[derive(Clone, Copy)]
pub(crate) struct TraceFileStdout {
    pub tee_stdout: bool,
    pub stream_iterable_closed: Option<super::IterableClosedStream>,
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
    kind: Option<SessionUpdateChunkKind>,
    stdout: TraceFileStdout,
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
    if let Some(warn) =
        super::operational_iterable_closed_for_emit(line, stdout.stream_iterable_closed)
    {
        if !writer.iterable_closed_warned {
            crate::output::print_log_warning(warn);
            writer.iterable_closed_warned = true;
        }
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
            tee_stdout: stdout.tee_stdout,
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
        for (kind, tl, stream) in coalesce.feed(kind, text) {
            trace_file_write_line(
                trace_file,
                &tl,
                Some(kind),
                TraceFileStdout {
                    tee_stdout: opts.tee_stdout,
                    stream_iterable_closed: stream,
                },
            )
            .await;
        }
        return;
    }
    for (kind, tl, stream) in coalesce.flush_all() {
        trace_file_write_line(
            trace_file,
            &tl,
            Some(kind),
            TraceFileStdout {
                tee_stdout: opts.tee_stdout,
                stream_iterable_closed: stream,
            },
        )
        .await;
    }
    let unparsed_tee = opts.tee_stdout && opts.parsed.is_none();
    trace_file_write_line(
        trace_file,
        opts.raw_line,
        None,
        TraceFileStdout {
            tee_stdout: unparsed_tee,
            stream_iterable_closed: None,
        },
    )
    .await;
}

#[cfg(test)]
mod trace_line_write_kiss {
    #[test]
    fn smoke_trace_line_write_symbol_names_for_kiss() {
        let _ = std::any::type_name::<super::ReaderTraceLineOpts>();
        let _ = std::any::type_name::<super::WriteTraceLineCoalescedOpts<'_>>();
        let _ = stringify!(
            reader_loop_verbose_and_trace_line,
            raw_output_suppress_thought_stdout,
            TraceFileStdout,
            TraceTeeStdoutCtx,
            trace_file_write_line,
            write_trace_line_coalesced
        );
    }
}
