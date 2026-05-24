use crate::acp::*;

#[test]
fn trace_chunk_coalescer_merges_two_small_message_chunks() {
    let mut c = TraceChunkCoalescer::default();
    assert!(c.feed(SessionUpdateChunkKind::Message, "hel").is_empty());
    assert!(c.feed(SessionUpdateChunkKind::Message, "lo").is_empty());
    let fin = c.flush_all();
    assert_eq!(fin.len(), 1);
    assert_eq!(
        fin[0],
        (SessionUpdateChunkKind::Message, "hello".to_string(), None)
    );
}

#[test]
fn trace_chunk_coalescer_feed_preserves_repeated_interleaved_order() {
    let mut c = TraceChunkCoalescer::default();
    assert!(c.feed(SessionUpdateChunkKind::Message, "m1").is_empty());
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Thought, "t1"),
        vec![(SessionUpdateChunkKind::Message, "m1".to_string(), None)]
    );
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Message, "m2"),
        vec![(SessionUpdateChunkKind::Thought, "t1".to_string(), None)]
    );
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Thought, "t2"),
        vec![(SessionUpdateChunkKind::Message, "m2".to_string(), None)]
    );
    assert_eq!(
        c.flush_all(),
        vec![(SessionUpdateChunkKind::Thought, "t2".to_string(), None)]
    );
}

#[test]
fn trace_chunk_coalescer_flush_all_preserves_interleaved_chunk_order_thought_then_message() {
    let mut c = TraceChunkCoalescer::default();
    assert!(c.feed(SessionUpdateChunkKind::Thought, "t").is_empty());
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Message, "m"),
        vec![(SessionUpdateChunkKind::Thought, "t".to_string(), None),]
    );
    assert_eq!(
        c.flush_all(),
        vec![(SessionUpdateChunkKind::Message, "m".to_string(), None)]
    );
}

#[test]
fn trace_chunk_coalescer_flush_all_preserves_interleaved_chunk_order_message_then_thought() {
    let mut c = TraceChunkCoalescer::default();
    assert!(c.feed(SessionUpdateChunkKind::Message, "m").is_empty());
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Thought, "t"),
        vec![(SessionUpdateChunkKind::Message, "m".to_string(), None),]
    );
    assert_eq!(
        c.flush_all(),
        vec![(SessionUpdateChunkKind::Thought, "t".to_string(), None)]
    );
}

#[test]
fn trace_chunk_coalescer_must_not_drop_consecutive_identical_lines() {
    let mut c = TraceChunkCoalescer::default();
    let out = c.feed(SessionUpdateChunkKind::Message, "yes\nyes\n");
    assert_eq!(
        out,
        vec![
            (SessionUpdateChunkKind::Message, "yes".to_string(), None),
            (SessionUpdateChunkKind::Message, "yes".to_string(), None),
        ],
        "consecutive identical lines must not be deduplicated"
    );
}

#[test]
fn trace_chunk_coalescer_emits_at_cap_like_verbose() {
    let max = ACP_VERBOSE_COALESCE_MAX;
    let mut c = TraceChunkCoalescer::default();
    let chunk = "x".repeat(max + 10);
    let out = c.feed(SessionUpdateChunkKind::Message, &chunk);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].0, SessionUpdateChunkKind::Message);
    assert_eq!(out[0].1.chars().count(), max);
    let fin = c.flush_all();
    assert_eq!(fin.len(), 1);
    assert_eq!(
        fin[0],
        (SessionUpdateChunkKind::Message, "x".repeat(10), None)
    );
}
