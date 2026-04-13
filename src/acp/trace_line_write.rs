//! Trace file line emission and coalesced writes (used by the ACP stdout reader).

use super::{
    PromptTraceWriter, VerboseTraceCoalesceState, session_update_chunk_parts,
    write_trace_line_coalesced,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

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

#[test]
fn kiss_stringify_trace_line_write() {
    let _ = stringify!(ReaderTraceLineOpts);
    let _ = stringify!(reader_loop_verbose_and_trace_line);
}
