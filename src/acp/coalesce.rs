// Verbose/trace coalescing for `session/update` chunks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        let hard_end = coalesce_char_boundary_at(buf, ACP_VERBOSE_COALESCE_MAX);
        let (emit_end, drain_end, drained_chars) =
            coalesce_word_split_points(buf, hard_end);
        let emitted = buf[..emit_end].to_string();
        buf.drain(..drain_end);
        *buf_chars -= drained_chars;
        emissions.push(emitted);
    }
}

fn coalesce_word_split_points(buf: &str, hard_end: usize) -> (usize, usize, usize) {
    let region = &buf[..hard_end];
    let mut last_space_start: Option<usize> = None;
    for (i, ch) in region.char_indices() {
        if ch == ' ' {
            last_space_start = Some(i);
        }
    }
    if let Some(space_start) = last_space_start {
        let mut drain_end = space_start;
        for ch in buf[space_start..hard_end].chars() {
            if ch == ' ' {
                drain_end += ch.len_utf8();
            } else {
                break;
            }
        }
        let emit_end = space_start;
        let emit_chars = buf[..emit_end].chars().count();
        if emit_chars > 0 {
            let drained_chars = buf[..drain_end].chars().count();
            return (emit_end, drain_end, drained_chars);
        }
    }
    (hard_end, hard_end, ACP_VERBOSE_COALESCE_MAX)
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
                Self::flush_if_nonempty(&mut self.thought, &mut self.thought_chars, "acp thought");
                Self::feed_buf(&mut self.message, &mut self.message_chars, chunk, "acp message");
            }
            SessionUpdateChunkKind::Thought => {
                Self::flush_if_nonempty(&mut self.message, &mut self.message_chars, "acp message");
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
pub(crate) fn session_update_chunk_parts(
    v: &Value,
) -> Option<(SessionUpdateChunkKind, &str)> {
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
        .unwrap_or("");
    Some((kind, text))
}

#[cfg(test)]
mod coalesce_tests {
    use super::{ACP_VERBOSE_COALESCE_MAX, coalesce_flush_cap};

    #[test]
    fn coalesce_kiss_stringify_units() {
        let _ = stringify!(coalesce_append_chunk);
        let _ = stringify!(coalesce_char_boundary_at);
        let _ = stringify!(coalesce_flush_nonempty);
        let _ = stringify!(session_update_chunk_parts);
        let _ = stringify!(SessionUpdateChunkKind);
    }

    #[test]
    fn coalesce_flush_cap_preserves_tab_boundary_content() {
        let original = format!("{}\t{}", "a".repeat(50), "b".repeat(100));
        let mut buf = original.clone();
        let mut buf_chars = buf.chars().count();
        let mut emissions = Vec::new();
        coalesce_flush_cap(&mut buf, &mut buf_chars, &mut emissions);
        assert_eq!(emissions.len(), 1);
        assert_eq!(emissions[0].chars().count(), ACP_VERBOSE_COALESCE_MAX);
        let rebuilt = format!("{}{}", emissions.concat(), buf);
        assert_eq!(rebuilt, original);
        assert!(rebuilt.contains('\t'));
    }
}
