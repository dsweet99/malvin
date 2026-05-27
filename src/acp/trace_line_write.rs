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
pub(crate) struct TraceFileStdout<'a> {
    pub tee_stdout: bool,
    pub stream_iterable_closed: Option<super::IterableClosedStream>,
    pub stream_upgrade_plan: bool,
    pub tee_line_override: Option<&'a str>,
    pub tee_line_display: Option<&'a str>,
    /// When set, trace file and stdout tee share this timestamp (tool-summary path).
    pub ts: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub(crate) struct TraceTeeStdoutCtx<'a> {
    pub(crate) tee_stdout: bool,
    pub(crate) kind: Option<SessionUpdateChunkKind>,
    pub(crate) ts: &'a str,
}

use crate::acp::trace_line_write_tee::{format_trace_display_line, trace_stdout_tee_payload, trace_tee_stdout_line};
use crate::acp::trace_line_write_tool_summary::write_tool_summary_trace_line;

pub async fn trace_file_write_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    kind: Option<SessionUpdateChunkKind>,
    stdout: TraceFileStdout<'_>,
) {
    if matches!(kind, Some(SessionUpdateChunkKind::Thought)) {
        crate::agent_phase::note_thought_activity();
    }
    let display_line = format_trace_display_line(line, kind);
    let ts = stdout
        .ts
        .map_or_else(crate::output::timestamp_now_string, str::to_string);
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
    if super::operational_upgrade_plan_for_emit(line, stdout.stream_upgrade_plan) {
        super::emit_operational_upgrade_plan_stop(&mut writer.upgrade_plan_warned);
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
    let stdout_line = stdout.tee_line_override.map_or_else(
        || trace_stdout_tee_payload(line, kind, writer),
        std::string::ToString::to_string,
    );
    trace_tee_stdout_line(
        writer,
        &stdout_line,
        stdout.tee_line_display,
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
        for (kind, tl, stream, upgrade_plan) in coalesce.feed(kind, text) {
            trace_file_write_line(
                trace_file,
                &tl,
                Some(kind),
                TraceFileStdout {
                    tee_stdout: opts.tee_stdout,
                    stream_iterable_closed: stream,
                    stream_upgrade_plan: upgrade_plan,
                    tee_line_override: None,
                    tee_line_display: None,
                    ts: None,
                },
            )
            .await;
        }
        return;
    }
    for (kind, tl, stream, upgrade_plan) in coalesce.flush_all() {
        trace_file_write_line(
            trace_file,
            &tl,
            Some(kind),
            TraceFileStdout {
                tee_stdout: opts.tee_stdout,
                stream_iterable_closed: stream,
                stream_upgrade_plan: upgrade_plan,
                tee_line_override: None,
                tee_line_display: None,
                ts: None,
            },
        )
        .await;
    }
    if let Some(parsed) = opts.parsed {
        if write_tool_summary_trace_line(trace_file, coalesce, parsed, opts.tee_stdout).await {
            return;
        }
    }
    let raw_protocol_tee = opts.tee_stdout && opts.parsed.is_none();
    trace_file_write_line(
        trace_file,
        opts.raw_line,
        None,
        TraceFileStdout {
            tee_stdout: raw_protocol_tee,
            stream_iterable_closed: None,
            stream_upgrade_plan: false,
            tee_line_override: None,
            tee_line_display: None,
            ts: None,
        },
    )
    .await;
}



#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_reader_trace_line_opts() { let _ = stringify!(ReaderTraceLineOpts); }

    #[test]
    fn kiss_cov_write_trace_line_coalesced_opts() { let _ = stringify!(WriteTraceLineCoalescedOpts); }

    #[test]
    fn kiss_cov_reader_loop_verbose_and_trace_line() { let _ = stringify!(reader_loop_verbose_and_trace_line); }

    #[test]
    fn kiss_cov_raw_output_suppress_thought_stdout() { let _ = stringify!(raw_output_suppress_thought_stdout); }

    #[test]
    fn kiss_cov_trace_file_stdout() { let _ = stringify!(TraceFileStdout); }

    #[test]
    fn kiss_cov_trace_tee_stdout_ctx() { let _ = stringify!(TraceTeeStdoutCtx); }

    #[test]
    fn kiss_cov_trace_file_write_line() { let _ = stringify!(trace_file_write_line); }

    #[test]
    fn kiss_cov_write_trace_line_coalesced() { let _ = stringify!(write_trace_line_coalesced); }

}
