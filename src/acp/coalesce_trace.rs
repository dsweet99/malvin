#[derive(Default)]
pub(crate) struct TraceChunkCoalescer {
    pub message: String,
    pub thought: String,
    message_chars: usize,
    thought_chars: usize,
    last_feed_kind: Option<SessionUpdateChunkKind>,
}

impl TraceChunkCoalescer {
    pub fn feed(&mut self, kind: SessionUpdateChunkKind, chunk: &str) -> Vec<(SessionUpdateChunkKind, String)> {
        self.last_feed_kind = Some(kind);
        let (buf, buf_chars) = match kind {
            SessionUpdateChunkKind::Message => (&mut self.message, &mut self.message_chars),
            SessionUpdateChunkKind::Thought => (&mut self.thought, &mut self.thought_chars),
        };
        let mut emissions = Vec::new();
        coalesce_append_chunk(buf, buf_chars, chunk, &mut emissions);
        emissions.into_iter().map(|line| (kind, line)).collect()
    }

    pub fn flush_all(&mut self) -> Vec<(SessionUpdateChunkKind, String)> {
        let mut out = Vec::new();
        let msg_empty = self.message.is_empty();
        let th_empty = self.thought.is_empty();
        if !msg_empty && !th_empty {
            match self.last_feed_kind {
                Some(SessionUpdateChunkKind::Message) => {
                    Self::flush_stream(
                        SessionUpdateChunkKind::Thought,
                        &mut self.thought,
                        &mut self.thought_chars,
                        &mut out,
                    );
                    Self::flush_stream(
                        SessionUpdateChunkKind::Message,
                        &mut self.message,
                        &mut self.message_chars,
                        &mut out,
                    );
                }
                Some(SessionUpdateChunkKind::Thought) | None => {
                    Self::flush_stream(
                        SessionUpdateChunkKind::Message,
                        &mut self.message,
                        &mut self.message_chars,
                        &mut out,
                    );
                    Self::flush_stream(
                        SessionUpdateChunkKind::Thought,
                        &mut self.thought,
                        &mut self.thought_chars,
                        &mut out,
                    );
                }
            }
        } else {
            Self::flush_stream(
                SessionUpdateChunkKind::Message,
                &mut self.message,
                &mut self.message_chars,
                &mut out,
            );
            Self::flush_stream(
                SessionUpdateChunkKind::Thought,
                &mut self.thought,
                &mut self.thought_chars,
                &mut out,
            );
        }
        out
    }

    fn flush_stream(
        kind: SessionUpdateChunkKind,
        buf: &mut String,
        buf_chars: &mut usize,
        out: &mut Vec<(SessionUpdateChunkKind, String)>,
    ) {
        if !buf.is_empty() {
            out.push((kind, std::mem::take(buf)));
            *buf_chars = 0;
        }
    }
}

pub(crate) struct VerboseTraceCoalesceState<'a> {
    pub verbose: &'a mut VerboseIoCoalescer,
    pub trace: &'a mut TraceChunkCoalescer,
}

#[test]
fn kiss_stringify_coalesce_b() {
    let _ = stringify!(TraceChunkCoalescer);
    let _ = stringify!(TraceChunkCoalescer::feed);
    let _ = stringify!(TraceChunkCoalescer::flush_all);
    let _ = stringify!(VerboseTraceCoalesceState);
}
