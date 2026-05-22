#[derive(Default)]
pub(crate) struct TraceChunkCoalescer {
    pub message: String,
    pub thought: String,
    message_chars: usize,
    thought_chars: usize,
    message_iterable_closed: Option<crate::acp::IterableClosedStream>,
    thought_iterable_closed: Option<crate::acp::IterableClosedStream>,
    pub tool_tracker: crate::acp::tool_summary::ToolSummaryTracker,
}

pub(crate) type TraceChunkEmission =
    (SessionUpdateChunkKind, String, Option<crate::acp::IterableClosedStream>);

impl TraceChunkCoalescer {
    pub fn feed(&mut self, kind: SessionUpdateChunkKind, chunk: &str) -> Vec<TraceChunkEmission> {
        let mut out = self.flush_other_stream(kind);
        let (buf, buf_chars) = match kind {
            SessionUpdateChunkKind::Message => (&mut self.message, &mut self.message_chars),
            SessionUpdateChunkKind::Thought => (&mut self.thought, &mut self.thought_chars),
        };
        let mut emissions = Vec::new();
        coalesce_append_chunk(buf, buf_chars, chunk, &mut emissions);
        let stream = crate::acp::iterable_closed_stream_from_buffer(buf);
        match kind {
            SessionUpdateChunkKind::Message => self.message_iterable_closed = stream,
            SessionUpdateChunkKind::Thought => self.thought_iterable_closed = stream,
        }
        out.extend(emissions.into_iter().map(|line| (kind, line, stream)));
        out
    }

    fn flush_other_stream(&mut self, kind: SessionUpdateChunkKind) -> Vec<TraceChunkEmission> {
        let mut out = Vec::new();
        match kind {
            SessionUpdateChunkKind::Message => {
                Self::flush_stream(FlushStreamCtx {
                    kind: SessionUpdateChunkKind::Thought,
                    buf: &mut self.thought,
                    buf_chars: &mut self.thought_chars,
                    out: &mut out,
                    iterable_closed: &mut self.thought_iterable_closed,
                });
            }
            SessionUpdateChunkKind::Thought => {
                Self::flush_stream(FlushStreamCtx {
                    kind: SessionUpdateChunkKind::Message,
                    buf: &mut self.message,
                    buf_chars: &mut self.message_chars,
                    out: &mut out,
                    iterable_closed: &mut self.message_iterable_closed,
                });
            }
        }
        out
    }

    pub fn flush_all(&mut self) -> Vec<TraceChunkEmission> {
        let mut out = Vec::new();
        Self::flush_stream(FlushStreamCtx {
            kind: SessionUpdateChunkKind::Message,
            buf: &mut self.message,
            buf_chars: &mut self.message_chars,
            out: &mut out,
            iterable_closed: &mut self.message_iterable_closed,
        });
        Self::flush_stream(FlushStreamCtx {
            kind: SessionUpdateChunkKind::Thought,
            buf: &mut self.thought,
            buf_chars: &mut self.thought_chars,
            out: &mut out,
            iterable_closed: &mut self.thought_iterable_closed,
        });
        out
    }

    fn flush_stream(ctx: FlushStreamCtx<'_>) {
        if !ctx.buf.is_empty() {
            let stream = *ctx.iterable_closed;
            ctx.out.push((ctx.kind, std::mem::take(ctx.buf), stream));
            *ctx.buf_chars = 0;
            *ctx.iterable_closed = None;
        }
    }
}

struct FlushStreamCtx<'a> {
    kind: SessionUpdateChunkKind,
    buf: &'a mut String,
    buf_chars: &'a mut usize,
    out: &'a mut Vec<TraceChunkEmission>,
    iterable_closed: &'a mut Option<crate::acp::IterableClosedStream>,
}

pub(crate) struct VerboseTraceCoalesceState<'a> {
    pub verbose: &'a mut VerboseIoCoalescer,
    pub trace: &'a mut TraceChunkCoalescer,
}

#[test]
fn trace_chunk_coalescer_feed_and_flush() {
    let _ = TraceChunkCoalescer::feed;
    let _: Option<VerboseTraceCoalesceState> = None;
    let mut coalescer = TraceChunkCoalescer::default();
    coalescer.feed(SessionUpdateChunkKind::Message, "hello");
    let flushed = coalescer.flush_all();
    assert!(flushed.iter().any(|(_, text, _)| text.contains("hello")));
}

#[test]
fn trace_chunk_coalescer_flush_other_stream_preserves_iterable_closed_on_flush_stream_ctx() {
    let _ = stringify!(FlushStreamCtx);
    let mut coalescer = TraceChunkCoalescer::default();
    coalescer.feed(
        SessionUpdateChunkKind::Message,
        "Error: T: WritableIterable is closed",
    );
    let switched = coalescer.feed(SessionUpdateChunkKind::Thought, "x");
    assert!(switched.iter().any(|(k, _, stream)| {
        *k == SessionUpdateChunkKind::Message
            && *stream == Some(crate::acp::IterableClosedStream::Writable)
    }));
}

#[test]
fn trace_chunk_coalescer_iterable_closed_flag_survives_split_feed() {
    let mut coalescer = TraceChunkCoalescer::default();
    let part_a = "Error: T: WritableIter";
    let part_b = "able is closed";
    let mid = coalescer.feed(SessionUpdateChunkKind::Message, part_a);
    assert!(mid.iter().all(|(_, _, stream)| stream.is_none()));
    coalescer.feed(SessionUpdateChunkKind::Message, part_b);
    let flushed = coalescer.flush_all();
    assert!(
        flushed
            .iter()
            .any(|(_, _, stream)| *stream == Some(crate::acp::IterableClosedStream::Writable))
    );
}
