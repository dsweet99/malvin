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

fn trace_tee_stdout_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    tee_stdout: bool,
    kind: Option<SessionUpdateChunkKind>,
) {
    if !tee_stdout {
        return;
    }
    if raw_output_should_skip_chunk(kind, writer) {
        return;
    }
    if writer.raw_output {
        println!("{line}");
        return;
    }
    match writer.stdout_replacement {
        Some(rep) => {
            if !writer.placeholder_emitted {
                crate::output::print_stdout_acp_tee_line(
                    crate::output::AcpTeeDirection::FromAgent,
                    &writer.who,
                    rep,
                );
                writer.placeholder_emitted = true;
            }
        }
        None => crate::output::print_stdout_acp_tee_line(
            crate::output::AcpTeeDirection::FromAgent,
            &writer.who,
            line,
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
    let formatted = crate::output::format_line(&writer.who, line);
    if let Err(e) = writer.file.write_all(formatted.as_bytes()).await {
        warn!(error = %e, "trace write failed");
        return;
    }
    if let Err(e) = writer.file.write_all(b"\n").await {
        warn!(error = %e, "trace newline failed");
        return;
    }
    trace_tee_stdout_line(writer, line, tee_stdout, kind);
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
fn kiss_stringify_trace_line_write() {
    let _ = stringify!(ReaderTraceLineOpts);
    let _ = stringify!(reader_loop_verbose_and_trace_line);
    let _ = stringify!(trace_file_write_line);
    let _ = stringify!(write_trace_line_coalesced);
}
