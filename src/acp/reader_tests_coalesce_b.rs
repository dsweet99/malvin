use crate::acp::*;

#[test]
fn coalesce_char_boundary_at_past_end_yields_len() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    assert_eq!(coalesce_char_boundary_at("hi", 99), 2);
    assert_eq!(coalesce_char_boundary_at("", 1), 0);
    let xs = "x".repeat(max);
    assert_eq!(coalesce_char_boundary_at(&xs, max), xs.len());
}

#[test]
fn coalesce_flush_cap_drains_exactly_cap_char_buffer() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = "x".repeat(max);
    let mut buf_chars = buf.chars().count();
    let mut out = Vec::new();
    coalesce_flush_cap(&mut buf, &mut buf_chars, &mut out);
    assert_eq!(out, vec!["x".repeat(max)]);
    assert!(buf.is_empty());
}

#[test]
fn coalesce_flush_cap_multiple_iterations() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = "y".repeat(max * 3 + 10);
    let mut buf_chars = buf.chars().count();
    let mut out = Vec::new();
    coalesce_flush_cap(&mut buf, &mut buf_chars, &mut out);
    assert_eq!(out.len(), 3);
    assert_eq!(buf.len(), 10);
}

#[test]
fn coalesce_flush_nonempty_direct() {
    let mut buf = String::from("hello");
    let mut buf_chars = buf.chars().count();
    let mut out = Vec::new();
    coalesce_flush_nonempty(&mut buf, &mut buf_chars, &mut out);
    assert_eq!(out, vec!["hello".to_string()]);
    assert!(buf.is_empty());
    coalesce_flush_nonempty(&mut buf, &mut buf_chars, &mut out);
    assert_eq!(out.len(), 1);
}

#[test]
fn coalesce_append_splits_on_unicode_scalar_count() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let s = "€".repeat(max + 5);
    coalesce_append_chunk(&mut buf, &mut buf_chars, &s, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].chars().count(), max);
    assert_eq!(buf.chars().count(), 5);
}

#[test]
fn coalesce_flush_cap_splits_at_word_boundary() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let word = "abcdefghij ";
    let repeated = word.repeat(max);
    coalesce_append_chunk(&mut buf, &mut buf_chars, &repeated, &mut out);
    assert!(!out.is_empty(), "should have emitted at least one segment");
    for segment in &out {
        for w in segment.split_whitespace() {
            assert_eq!(w, "abcdefghij", "word should not be split: {w:?}");
        }
    }
    for w in buf.split_whitespace() {
        assert_eq!(
            w, "abcdefghij",
            "remainder should not contain partial words: {w:?}"
        );
    }
}

#[test]
fn verbose_io_coalescer_feed_and_flush_all_covers_paths() {
    let mut c = VerboseIoCoalescer::default();
    c.feed(SessionUpdateChunkKind::Message, "hello");
    c.feed(SessionUpdateChunkKind::Thought, "think");
    c.flush_all();
    assert!(c.message.is_empty(), "message buffer should flush");
    assert!(c.thought.is_empty(), "thought buffer should flush");
}

#[test]
fn verbose_io_coalescer_switch_flushes_previous_kind_buffer() {
    let mut c = VerboseIoCoalescer::default();
    c.feed(SessionUpdateChunkKind::Message, "m1");
    assert_eq!(c.message, "m1");
    assert!(c.thought.is_empty());
    c.feed(SessionUpdateChunkKind::Thought, "t1");
    assert!(
        c.message.is_empty(),
        "message buffer should flush on kind switch"
    );
    assert_eq!(c.thought, "t1");
    c.feed(SessionUpdateChunkKind::Message, "m2");
    assert_eq!(c.message, "m2");
    assert!(
        c.thought.is_empty(),
        "thought buffer should flush on kind switch"
    );
}

#[test]
fn coalesce_flush_cap_emissions_scale_linearly_with_input_size() {
    let cap = ACP_VERBOSE_COALESCE_MAX;
    let emission_count = |units: usize| {
        let n = cap * units;
        let mut buf = "a".repeat(n);
        let mut buf_chars = buf.chars().count();
        let mut emissions = Vec::new();
        coalesce_flush_cap(&mut buf, &mut buf_chars, &mut emissions);
        emissions.len()
    };
    let e_small = emission_count(500);
    let e_large = emission_count(1000);
    assert!(
        e_small == 500 && e_large == 1000,
        "unexpected emission counts for fixed-size flush: small={e_small}, large={e_large}"
    );
}
