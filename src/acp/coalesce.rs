// Verbose/trace coalescing for `session/update` chunks.
#[derive(Clone, Copy)]
pub(crate) enum SessionUpdateChunkKind {
    Message,
    Thought,
}

/// Chunk text coalescing for **verbose** logs and **JSONL traces**: append until this many Unicode scalars,
/// a newline run, or a non-chunk line (JSON-RPC response, `tool_call`, etc.) triggers a flush.
pub(crate) const ACP_VERBOSE_COALESCE_MAX: usize = 125;

pub(crate) fn coalesce_append_chunk(
    buf: &mut String,
    buf_chars: &mut usize,
    chunk: &str,
    emissions: &mut Vec<String>,
) {
    let mut pos = 0usize;
    let b = chunk.as_bytes();
    while pos < b.len() {
        if let Some(rel) = b[pos..].iter().position(|&c| c == b'\n') {
            let end = pos + rel;
            let piece = &chunk[pos..end];
            buf.push_str(piece);
            *buf_chars += piece.chars().count();
            coalesce_flush_cap(buf, buf_chars, emissions);
            coalesce_flush_nonempty(buf, buf_chars, emissions);
            pos = end;
            while pos < b.len() && b[pos] == b'\n' {
                pos += 1;
            }
        } else {
            let piece = &chunk[pos..];
            buf.push_str(piece);
            *buf_chars += piece.chars().count();
            coalesce_flush_cap(buf, buf_chars, emissions);
            break;
        }
    }
}

pub(crate) fn coalesce_char_boundary_at(s: &str, n_chars: usize) -> usize {
    s.char_indices()
        .nth(n_chars)
        .map_or(s.len(), |(i, _)| i)
}

pub(crate) fn coalesce_flush_cap(buf: &mut String, buf_chars: &mut usize, emissions: &mut Vec<String>) {
    while *buf_chars >= ACP_VERBOSE_COALESCE_MAX {
        let end = coalesce_char_boundary_at(buf, ACP_VERBOSE_COALESCE_MAX);
        emissions.push(buf.drain(..end).collect());
        *buf_chars -= ACP_VERBOSE_COALESCE_MAX;
    }
}

pub(crate) fn coalesce_flush_nonempty(buf: &mut String, buf_chars: &mut usize, emissions: &mut Vec<String>) {
    if !buf.is_empty() {
        emissions.push(std::mem::take(buf));
        *buf_chars = 0;
    }
}

#[derive(Default)]
pub(crate) struct VerboseIoCoalescer {
    pub message: String,
    pub thought: String,
    message_chars: usize,
    thought_chars: usize,
}

impl VerboseIoCoalescer {
    pub fn feed(&mut self, kind: SessionUpdateChunkKind, chunk: &str) {
        match kind {
            SessionUpdateChunkKind::Message => {
                Self::feed_buf(&mut self.message, &mut self.message_chars, chunk, "acp message");
            }
            SessionUpdateChunkKind::Thought => {
                Self::feed_buf(&mut self.thought, &mut self.thought_chars, chunk, "acp thought");
            }
        }
    }

    pub fn flush_all(&mut self) {
        Self::flush_if_nonempty(&mut self.message, &mut self.message_chars, "acp message");
        Self::flush_if_nonempty(&mut self.thought, &mut self.thought_chars, "acp thought");
    }

    fn feed_buf(buf: &mut String, buf_chars: &mut usize, chunk: &str, label: &'static str) {
        let mut emissions = Vec::new();
        coalesce_append_chunk(buf, buf_chars, chunk, &mut emissions);
        for piece in emissions {
            info!(target: "malvin::acp::io", "{} {}", label, piece);
        }
    }

    fn flush_if_nonempty(buf: &mut String, buf_chars: &mut usize, label: &'static str) {
        if !buf.is_empty() {
            let piece = std::mem::take(buf);
            *buf_chars = 0;
            info!(target: "malvin::acp::io", "{} {}", label, piece);
        }
    }
}

/// `session/update` streaming chunks (`agent_message_chunk`, `agent_thought_chunk`).
pub(crate) fn session_update_chunk_parts(v: &Value) -> Option<(SessionUpdateChunkKind, String)> {
    if v.get("method").and_then(Value::as_str) != Some("session/update") {
        return None;
    }
    let update = v.pointer("/params/update")?;
    let kind = match update.get("sessionUpdate").and_then(Value::as_str)? {
        "agent_message_chunk" => SessionUpdateChunkKind::Message,
        "agent_thought_chunk" => SessionUpdateChunkKind::Thought,
        _ => return None,
    };
    let text = update
        .pointer("/content/text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Some((kind, text))
}

#[derive(Default)]
pub(crate) struct TraceChunkCoalescer {
    pub message: String,
    pub thought: String,
    message_chars: usize,
    thought_chars: usize,
}

impl TraceChunkCoalescer {
    pub fn feed(&mut self, kind: SessionUpdateChunkKind, chunk: &str) -> Vec<String> {
        let (buf, buf_chars) = match kind {
            SessionUpdateChunkKind::Message => (&mut self.message, &mut self.message_chars),
            SessionUpdateChunkKind::Thought => (&mut self.thought, &mut self.thought_chars),
        };
        let mut emissions = Vec::new();
        coalesce_append_chunk(buf, buf_chars, chunk, &mut emissions);
        emissions
    }

    pub fn flush_all(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        Self::flush_stream(&mut self.message, &mut self.message_chars, &mut out);
        Self::flush_stream(&mut self.thought, &mut self.thought_chars, &mut out);
        out
    }

    fn flush_stream(buf: &mut String, buf_chars: &mut usize, out: &mut Vec<String>) {
        if !buf.is_empty() {
            out.push(std::mem::take(buf));
            *buf_chars = 0;
        }
    }
}

fn trace_tee_stdout_line(writer: &mut PromptTraceWriter, line: &str, tee_stdout: bool) {
    if !tee_stdout {
        return;
    }
    match writer.stdout_replacement {
        Some(rep) => {
            if !writer.placeholder_emitted {
                crate::output::print_stdout_line(&writer.who, rep);
                writer.placeholder_emitted = true;
            }
        }
        None => crate::output::print_stdout_line(&writer.who, line),
    }
}

pub(crate) async fn trace_file_write_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    tee_stdout: bool,
) {
    let formatted = crate::output::format_line(&writer.who, line);
    if let Err(e) = writer.file.write_all(formatted.as_bytes()).await {
        warn!(error = %e, "trace write failed");
        return;
    }
    if let Err(e) = writer.file.write_all(b"\n").await {
        warn!(error = %e, "trace newline failed");
        return;
    }
    trace_tee_stdout_line(writer, line, tee_stdout);
}

pub(crate) async fn write_trace_line_coalesced(
    trace_file: &mut PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
    parsed: Option<&Value>,
    tee_stdout: bool,
) {
    if let Some((kind, text)) = parsed.and_then(session_update_chunk_parts) {
        for tl in coalesce.feed(kind, text.as_str()) {
            trace_file_write_line(trace_file, &tl, tee_stdout).await;
        }
        return;
    }
    for tl in coalesce.flush_all() {
        trace_file_write_line(trace_file, &tl, tee_stdout).await;
    }
}

pub(crate) struct VerboseTraceCoalesceState<'a> {
    pub verbose: &'a mut VerboseIoCoalescer,
    pub trace: &'a mut TraceChunkCoalescer,
}

#[test]
fn kiss_stringify_coalesce_a() {
    let _ = stringify!(SessionUpdateChunkKind);
    let _ = stringify!(ACP_VERBOSE_COALESCE_MAX);
    let _ = stringify!(coalesce_append_chunk);
    let _ = stringify!(coalesce_char_boundary_at);
    let _ = stringify!(coalesce_flush_cap);
    let _ = stringify!(coalesce_flush_nonempty);
    let _ = stringify!(VerboseIoCoalescer);
    let _ = stringify!(VerboseIoCoalescer::feed);
    let _ = stringify!(VerboseIoCoalescer::flush_all);
    let _ = stringify!(session_update_chunk_parts);
}

#[test]
fn kiss_stringify_coalesce_b() {
    let _ = stringify!(TraceChunkCoalescer);
    let _ = stringify!(TraceChunkCoalescer::feed);
    let _ = stringify!(TraceChunkCoalescer::flush_all);
    let _ = stringify!(trace_tee_stdout_line);
    let _ = stringify!(trace_file_write_line);
    let _ = stringify!(write_trace_line_coalesced);
    let _ = stringify!(VerboseTraceCoalesceState);
}
