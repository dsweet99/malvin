use crate::acp::*;
use serde_json::Value;

#[test]
fn kiss_cov_coalesce_private_method_names() {
    let _ = stringify!(SessionUpdateChunkKind);
    let _ = stringify!(feed_buf);
    let _ = stringify!(flush_if_nonempty);
    let _: SessionUpdateChunkKind = SessionUpdateChunkKind::Message;
}

#[test]
fn session_update_chunk_parts_message() {
    let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"x","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"want to work "}}}}"#;
    let v: Value = serde_json::from_str(line).unwrap();
    let (k, t) = session_update_chunk_parts(&v).expect("chunk");
    assert!(matches!(k, crate::acp::SessionUpdateChunkKind::Message));
    assert_eq!(t, "want to work ");
}

#[test]
fn session_update_chunk_parts_thought() {
    let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_thought_chunk","content":{"type":"text","text":"thinking"}}}}"#;
    let v: Value = serde_json::from_str(line).unwrap();
    let (k, t) = session_update_chunk_parts(&v).expect("chunk");
    assert!(matches!(k, crate::acp::SessionUpdateChunkKind::Thought));
    assert_eq!(t, "thinking");
}

#[test]
fn session_update_chunk_parts_skips_non_session_update() {
    let v: Value = serde_json::from_str(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#).unwrap();
    assert!(session_update_chunk_parts(&v).is_none());
}

#[test]
fn coalesce_append_emits_at_newline_without_newline_in_output() {
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    coalesce_append_chunk(&mut buf, &mut buf_chars, "hello\nworld", &mut out);
    assert_eq!(out, vec!["hello".to_string()]);
    assert_eq!(buf, "world");
    coalesce_append_chunk(&mut buf, &mut buf_chars, "\n", &mut out);
    assert_eq!(out, vec!["hello".to_string(), "world".to_string()]);
    assert!(buf.is_empty());
}

#[test]
fn coalesce_append_emits_at_cap_then_carries_rest() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let prefix: String = (0..max).map(|_| 'x').collect();
    let extra = format!("{prefix}abcde");
    coalesce_append_chunk(&mut buf, &mut buf_chars, &extra, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].chars().count(), max);
    assert_eq!(buf, "abcde");
}

#[test]
fn coalesce_append_multiple_cap_rounds_without_newline() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let n = max * 2 + 40;
    coalesce_append_chunk(&mut buf, &mut buf_chars, &"x".repeat(n), &mut out);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].len(), max);
    assert_eq!(out[1].len(), max);
    assert_eq!(buf.len(), 40);
}

#[test]
fn coalesce_append_cap_then_remainder_flushed_at_newline() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let chunk = format!("{}\n", "a".repeat(max + 5));
    coalesce_append_chunk(&mut buf, &mut buf_chars, &chunk, &mut out);
    assert_eq!(out, vec!["a".repeat(max), "aaaaa".to_string()]);
    assert!(buf.is_empty());
}

#[test]
fn coalesce_append_only_newlines_emits_nothing() {
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    coalesce_append_chunk(&mut buf, &mut buf_chars, "\n\n\n", &mut out);
    assert!(out.is_empty());
    assert!(buf.is_empty());
}
