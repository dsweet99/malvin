use crate::acp::ResponseTx;
use crate::acp::*;
use serde_json::{Value, json};
use std::collections::HashMap;
#[cfg(unix)]
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::process::Command;
use tokio::sync::oneshot;
use tokio::sync::{Mutex, Notify};

fn acp_activity_state() -> (Arc<AtomicU64>, Arc<Notify>) {
    (Arc::new(AtomicU64::new(0)), Arc::new(Notify::new()))
}

#[cfg(unix)]
const CAT_BIN: &str = "cat";

#[cfg(unix)]
fn unix_bin_with_fallback(name: &str) -> String {
    let bin = format!("/bin/{name}");
    if Path::new(&bin).is_file() {
        return bin;
    }
    let usr_bin = format!("/usr/bin/{name}");
    if Path::new(&usr_bin).is_file() {
        return usr_bin;
    }
    name.to_string()
}

#[tokio::test]
async fn test_dispatch_response_ok_error_orphans_and_malformed() {
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));

    let (tx, rx) = oneshot::channel();
    pending.lock().await.insert(1, tx);
    let ok = json!({"jsonrpc": "2.0", "id": 1, "result": {"a": 1}});
    assert!(dispatch_response(&ok, &pending, None).await);
    assert_eq!(rx.await.unwrap().unwrap()["a"], 1);

    let (tx2, rx2) = oneshot::channel();
    pending.lock().await.insert(2, tx2);
    let err = json!({"jsonrpc": "2.0", "id": 2, "error": {"message": "e"}});
    assert!(dispatch_response(&err, &pending, None).await);
    assert!(rx2.await.unwrap().unwrap_err().contains("message"));

    let (tx3, rx3) = oneshot::channel();
    pending.lock().await.insert(3, tx3);
    let neither = json!({"jsonrpc": "2.0", "id": 3});
    assert!(dispatch_response(&neither, &pending, None).await);
    assert!(
        rx3.await
            .unwrap()
            .unwrap_err()
            .contains("missing result/error")
    );

    let no_id = json!({"jsonrpc": "2.0", "result": {}});
    assert!(!dispatch_response(&no_id, &pending, None).await);

    let bad_id = json!({"jsonrpc": "2.0", "id": "x", "result": {}});
    assert!(!dispatch_response(&bad_id, &pending, None).await);

    let orphan = json!({"jsonrpc": "2.0", "id": 99, "result": {}});
    assert!(dispatch_response(&orphan, &pending, None).await);
}

/// JSON-RPC 2.0 allows `id` to be a JSON number; serde may represent small integers as `i64`.
#[tokio::test]
async fn dispatch_resolves_pending_when_response_id_is_i64() {
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = oneshot::channel();
    pending.lock().await.insert(7, tx);
    let msg = json!({"jsonrpc": "2.0", "id": 7i64, "result": {"v": 1}});
    assert!(dispatch_response(&msg, &pending, None).await);
    assert_eq!(rx.await.unwrap().unwrap()["v"], 1);
}

/// JSON-RPC 2.0 allows `id` to be a string. Peers may echo a numeric request id as a string in the response.
#[tokio::test]
async fn dispatch_resolves_pending_when_response_id_is_decimal_string() {
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = oneshot::channel();
    pending.lock().await.insert(1, tx);
    let msg = json!({"jsonrpc": "2.0", "id": "1", "result": {"v": 42}});
    assert!(
        dispatch_response(&msg, &pending, None).await,
        "string id should match pending request 1"
    );
    assert_eq!(rx.await.unwrap().unwrap()["v"], 42);
}

#[cfg(unix)]
#[tokio::test]
async fn test_handle_incoming_line_parse_error_and_extension_method() {
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let mut child = Command::new(unix_bin_with_fallback("sleep"))
        .arg("8")
        .stdin(Stdio::piped())
        .spawn()
        .expect("sleep");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let _reap = tokio::spawn(async move {
        let _ = child.kill().await;
        let _ = child.wait().await;
    });

    handle_incoming_line(
        "%%%",
        IncomingLineDispatch {
            pending: &pending,
            stdin: &stdin,
            acp_activity_seq: &acp_activity_seq,
            acp_activity_notify: &acp_activity_notify,
            prompt_cleanup: None,
            acp_verbose: false,
        },
    )
    .await;
    handle_incoming_line(
        r#"{"jsonrpc":"2.0","method":"cursor/task","params":{}}"#,
        IncomingLineDispatch {
            pending: &pending,
            stdin: &stdin,
            acp_activity_seq: &acp_activity_seq,
            acp_activity_notify: &acp_activity_notify,
            prompt_cleanup: None,
            acp_verbose: false,
        },
    )
    .await;
    assert_eq!(
        acp_activity_seq.load(Ordering::SeqCst),
        1,
        "only valid JSON should count as ACP activity"
    );
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
fn trace_chunk_coalescer_merges_two_small_message_chunks() {
    let mut c = TraceChunkCoalescer::default();
    assert!(c.feed(SessionUpdateChunkKind::Message, "hel").is_empty());
    assert!(c.feed(SessionUpdateChunkKind::Message, "lo").is_empty());
    let fin = c.flush_all();
    assert_eq!(fin.len(), 1);
    assert_eq!(
        fin[0],
        (SessionUpdateChunkKind::Message, "hello".to_string())
    );
}

#[test]
fn trace_chunk_coalescer_feed_preserves_repeated_interleaved_order() {
    let mut c = TraceChunkCoalescer::default();
    assert!(c.feed(SessionUpdateChunkKind::Message, "m1").is_empty());
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Thought, "t1"),
        vec![(SessionUpdateChunkKind::Message, "m1".to_string())]
    );
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Message, "m2"),
        vec![(SessionUpdateChunkKind::Thought, "t1".to_string())]
    );
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Thought, "t2"),
        vec![(SessionUpdateChunkKind::Message, "m2".to_string())]
    );
    assert_eq!(
        c.flush_all(),
        vec![(SessionUpdateChunkKind::Thought, "t2".to_string())]
    );
}

#[test]
fn trace_chunk_coalescer_flush_all_preserves_interleaved_chunk_order_thought_then_message() {
    let mut c = TraceChunkCoalescer::default();
    assert!(c.feed(SessionUpdateChunkKind::Thought, "t").is_empty());
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Message, "m"),
        vec![(SessionUpdateChunkKind::Thought, "t".to_string()),]
    );
    assert_eq!(
        c.flush_all(),
        vec![(SessionUpdateChunkKind::Message, "m".to_string())]
    );
}

#[test]
fn trace_chunk_coalescer_flush_all_preserves_interleaved_chunk_order_message_then_thought() {
    let mut c = TraceChunkCoalescer::default();
    assert!(c.feed(SessionUpdateChunkKind::Message, "m").is_empty());
    assert_eq!(
        c.feed(SessionUpdateChunkKind::Thought, "t"),
        vec![(SessionUpdateChunkKind::Message, "m".to_string()),]
    );
    assert_eq!(
        c.flush_all(),
        vec![(SessionUpdateChunkKind::Thought, "t".to_string())]
    );
}

#[test]
fn trace_chunk_coalescer_must_not_drop_consecutive_identical_lines() {
    let mut c = TraceChunkCoalescer::default();
    let out = c.feed(SessionUpdateChunkKind::Message, "yes\nyes\n");
    assert_eq!(
        out,
        vec![
            (SessionUpdateChunkKind::Message, "yes".to_string()),
            (SessionUpdateChunkKind::Message, "yes".to_string()),
        ],
        "consecutive identical lines must not be deduplicated"
    );
}

#[tokio::test]
async fn write_trace_line_coalesced_writes_non_chunk_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("coalesce-trace.log");
    let f = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file: f,
        who: "kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
    };
    let mut c = TraceChunkCoalescer::default();
    let parsed = serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"ok":true}});
    super::trace_line_write::write_trace_line_coalesced(
        &mut writer,
        &mut c,
        super::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: Some(&parsed),
            raw_line: r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#,
            tee_stdout: false,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains(r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#),
        "non-chunk ACP lines should be preserved in trace output"
    );
}

#[tokio::test]
async fn write_trace_line_coalesced_does_not_tee_parsed_non_chunk_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("coalesce-trace-no-tee.log");
    let f = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file: f,
        who: "kpop".to_string(),
        plain_lines: false,
        stdout_replacement: Some("<suppressed>"),
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
    };
    let mut c = TraceChunkCoalescer::default();
    let parsed = serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"ok":true}});
    super::trace_line_write::write_trace_line_coalesced(
        &mut writer,
        &mut c,
        super::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: Some(&parsed),
            raw_line: r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#,
            tee_stdout: true,
        },
    )
    .await;
    assert!(
        !writer.placeholder_emitted,
        "parsed non-chunk ACP protocol lines must not be tee'd to stdout"
    );
}

#[tokio::test]
async fn write_trace_line_coalesced_writes_malformed_non_json_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("coalesce-trace-malformed.log");
    let f = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file: f,
        who: "kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
    };
    let mut c = TraceChunkCoalescer::default();
    super::trace_line_write::write_trace_line_coalesced(
        &mut writer,
        &mut c,
        super::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: None,
            raw_line: "not-json {{{",
            tee_stdout: false,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains("not-json {{{"),
        "malformed non-JSON ACP lines should still be preserved in trace output"
    );
}

#[tokio::test]
async fn trace_file_write_line_prefixes_with_prompt_who() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-prefix.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "review_1".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
    };
    crate::acp::trace_file_write_line(&mut writer, "hello", false, None).await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    let inner = crate::output::format_log_tag_inner("review_1");
    assert!(
        s.contains(&format!(":[{inner}]: hello\n")),
        "expected prompt-prefixed trace line, got {s:?}"
    );
}

#[tokio::test]
async fn raw_trace_file_write_line_records_thought_chunks_suppresses_thought_stdout_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-raw-thought.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "raw".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "internal reasoning",
        false,
        Some(SessionUpdateChunkKind::Thought),
    )
    .await;
    crate::acp::trace_file_write_line(
        &mut writer,
        "final answer",
        false,
        Some(SessionUpdateChunkKind::Message),
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains("[internal reasoning]"),
        "trace file should record thought chunks when raw_output, got {s:?}"
    );
    assert!(
        s.contains("final answer"),
        "raw output should keep message chunks, got {s:?}"
    );
}

#[tokio::test]
async fn trace_file_write_line_plain_mode_omits_tag_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-plain.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "<do".to_string(),
        plain_lines: true,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "assistant response",
        false,
        Some(SessionUpdateChunkKind::Message),
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert_eq!(s, "assistant response\n");
}

#[tokio::test]
async fn trace_file_write_line_brackets_thought_chunks_in_trace_output() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-thought.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "review_1".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "internal reasoning",
        false,
        Some(SessionUpdateChunkKind::Thought),
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains("[internal reasoning]"),
        "thought chunks should be bracketed in traces, got {s:?}"
    );
}

#[tokio::test]
async fn trace_file_write_line_stdout_markdown_flag_tees_without_panic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-md-tee.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "<kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "**x**",
        true,
        Some(SessionUpdateChunkKind::Message),
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains("**x**"),
        "trace file keeps raw markdown regardless of stdout markdown flag: {s:?}"
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
    assert_eq!(fin[0], (SessionUpdateChunkKind::Message, "x".repeat(10)));
}

#[test]
fn jsonrpc_response_id_parses_u64_and_decimal_string_and_rejects_garbage() {
    assert_eq!(jsonrpc_response_id_as_u64(&json!(42u64)), Some(42));
    assert_eq!(jsonrpc_response_id_as_u64(&json!(42i64)), Some(42));
    assert_eq!(jsonrpc_response_id_as_u64(&json!("99")), Some(99));
    assert_eq!(jsonrpc_response_id_as_u64(&json!("not-a-number")), None);
    assert_eq!(jsonrpc_response_id_as_u64(&json!(-1i64)), None);
    assert_eq!(jsonrpc_response_id_as_u64(&json!(null)), None);
}

#[test]
fn request_permission_correlation_id_top_level_params_and_request_id() {
    let top = json!({"jsonrpc":"2.0","id":1,"params":{"id":2}});
    assert_eq!(request_permission_correlation_id(&top), top.get("id"));
    let nested = json!({"jsonrpc":"2.0","method":"session/request_permission","params":{"id":2}});
    assert_eq!(
        request_permission_correlation_id(&nested),
        nested.pointer("/params/id")
    );
    let req_id = json!({"params":{"requestId":"9"}});
    assert_eq!(
        request_permission_correlation_id(&req_id),
        req_id.pointer("/params/requestId")
    );
    let none = json!({"method":"session/request_permission","params":{}});
    assert_eq!(request_permission_correlation_id(&none), None);
}

#[test]
fn test_permission_reply_shape() {
    let id = json!(42u64);
    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "outcome": { "outcome": "selected", "optionId": "allow-always" }
        }
    });
    assert!(body.get("result").is_some());
}

#[cfg(unix)]
#[tokio::test]
async fn test_handle_session_update_and_permission_replies() {
    use tokio::io::AsyncReadExt;

    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let mut child = Command::new(unix_bin_with_fallback(CAT_BIN))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cat");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let mut stdout = child.stdout.take().expect("stdout");

    handle_incoming_line(
        r#"{"jsonrpc":"2.0","method":"session/update","params":{"t":1}}"#,
        IncomingLineDispatch {
            pending: &pending,
            stdin: &stdin,
            acp_activity_seq: &acp_activity_seq,
            acp_activity_notify: &acp_activity_notify,
            prompt_cleanup: None,
            acp_verbose: false,
        },
    )
    .await;

    handle_incoming_line(
        r#"{"jsonrpc":"2.0","id":42,"method":"session/request_permission","params":{}}"#,
        IncomingLineDispatch {
            pending: &pending,
            stdin: &stdin,
            acp_activity_seq: &acp_activity_seq,
            acp_activity_notify: &acp_activity_notify,
            prompt_cleanup: None,
            acp_verbose: false,
        },
    )
    .await;

    drop(stdin);
    let mut received = Vec::new();
    stdout
        .read_to_end(&mut received)
        .await
        .expect("read stdout");
    let _ = child.wait().await.expect("wait cat");
    let line = String::from_utf8_lossy(&received);
    assert!(
        line.contains("allow-always")
            && (line.contains(r#""id":42"#) || line.contains(r#""id": 42"#)),
        "expected allow-always reply echoing id 42; got {line:?}"
    );
    assert_eq!(
        acp_activity_seq.load(Ordering::SeqCst),
        2,
        "both JSON messages should count as ACP activity"
    );
}

/// KPOP: `session/request_permission` with no correlation id anywhere still skips `write_rpc_line`.
#[cfg(unix)]
#[tokio::test]
async fn kpop_permission_without_correlation_id_writes_nothing_to_child_stdin() {
    use tokio::io::AsyncReadExt;

    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let mut child = Command::new(unix_bin_with_fallback(CAT_BIN))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cat");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let mut stdout = child.stdout.take().expect("stdout");

    handle_incoming_line(
        r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{}}"#,
        IncomingLineDispatch {
            pending: &pending,
            stdin: &stdin,
            acp_activity_seq: &acp_activity_seq,
            acp_activity_notify: &acp_activity_notify,
            prompt_cleanup: None,
            acp_verbose: false,
        },
    )
    .await;

    drop(stdin);
    let mut received = Vec::new();
    stdout
        .read_to_end(&mut received)
        .await
        .expect("read stdout");
    let _ = child.wait().await.expect("wait cat");
    assert!(
        received.is_empty(),
        "expected no bytes written for permission message without id; got {:?}",
        String::from_utf8_lossy(&received)
    );
}

/// Permission prompt with `id` only under `params` must still get an allow-always JSON-RPC reply line.
#[cfg(unix)]
#[tokio::test]
async fn permission_with_id_in_params_writes_allow_always_reply_line() {
    use tokio::io::AsyncReadExt;

    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let mut child = Command::new(unix_bin_with_fallback(CAT_BIN))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cat");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let mut stdout = child.stdout.take().expect("stdout");

    handle_incoming_line(
        r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{"id":77}}"#,
        IncomingLineDispatch {
            pending: &pending,
            stdin: &stdin,
            acp_activity_seq: &acp_activity_seq,
            acp_activity_notify: &acp_activity_notify,
            prompt_cleanup: None,
            acp_verbose: false,
        },
    )
    .await;

    drop(stdin);
    let mut received = Vec::new();
    stdout
        .read_to_end(&mut received)
        .await
        .expect("read stdout");
    let _ = child.wait().await.expect("wait cat");
    let line = String::from_utf8_lossy(&received);
    assert!(
        line.contains("allow-always")
            && (line.contains(r#""id":77"#) || line.contains(r#""id": 77"#)),
        "expected allow-always reply echoing id 77; got {line:?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn test_permission_json_or_write_failure_is_logged() {
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let mut child = Command::new(unix_bin_with_fallback("true"))
        .stdin(Stdio::piped())
        .spawn()
        .expect("true");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let _ = child.wait().await;
    handle_incoming_line(
        r#"{"jsonrpc":"2.0","id":9,"method":"session/request_permission","params":{}}"#,
        IncomingLineDispatch {
            pending: &pending,
            stdin: &stdin,
            acp_activity_seq: &acp_activity_seq,
            acp_activity_notify: &acp_activity_notify,
            prompt_cleanup: None,
            acp_verbose: false,
        },
    )
    .await;
    assert_eq!(
        acp_activity_seq.load(Ordering::SeqCst),
        1,
        "permission request should count as ACP activity even when reply write fails"
    );
    assert!(
        pending.lock().await.is_empty(),
        "permission write failure must not leak pending RPC state"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn test_reader_loop_drains_pending_on_stdout_eof() {
    let mut child = Command::new(unix_bin_with_fallback("true"))
        .stdout(Stdio::piped())
        .spawn()
        .expect("true");
    let stdout = child.stdout.take().expect("stdout");
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = oneshot::channel();
    pending.lock().await.insert(7, tx);
    let mut stdin_holder = Command::new(unix_bin_with_fallback("sleep"))
        .arg("1")
        .stdin(Stdio::piped())
        .spawn()
        .expect("sleep");
    let stdin = Arc::new(Mutex::new(stdin_holder.stdin.take().expect("stdin")));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let reader_dead = Arc::new(AtomicBool::new(false));
    let trace_writer: Arc<Mutex<Option<PromptTraceWriter>>> = Arc::new(Mutex::new(None));
    let busy = Arc::new(AtomicBool::new(false));
    let prompt_rpc_id = Arc::new(AtomicU64::new(0));
    let prompt_cleanup = Arc::new(PromptRpcCleanup {
        busy,
        trace_writer: trace_writer.clone(),
        prompt_rpc_id,
        idle_notify: None,
    });
    let waiter = spawn_acp_stdout_reader(ReaderSpawnArgs {
        stdout,
        pending: pending.clone(),
        stdin,
        acp_activity_seq,
        acp_activity_notify,
        reader_dead,
        trace_writer,
        prompt_cleanup,
        acp_verbose: false,
        tee_trace_stdout: false,
    });
    let err = rx.await.unwrap().unwrap_err();
    assert!(err.contains("closed") || err.contains("acp"));
    let _: () = waiter.await.unwrap();
    let _ = child.wait().await;
    let _ = stdin_holder.kill().await;
}

#[tokio::test]
async fn dispatch_clears_prompt_cleanup_when_id_matches() {
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let busy = Arc::new(AtomicBool::new(true));
    let trace_writer: Arc<Mutex<Option<PromptTraceWriter>>> = Arc::new(Mutex::new(None));
    let prompt_rpc_id = Arc::new(AtomicU64::new(5));
    let cleanup = PromptRpcCleanup {
        busy: busy.clone(),
        trace_writer: trace_writer.clone(),
        prompt_rpc_id: prompt_rpc_id.clone(),
        idle_notify: None,
    };
    let (tx, rx) = oneshot::channel();
    pending.lock().await.insert(5, tx);
    let msg = json!({"jsonrpc": "2.0", "id": 5, "result": {"stopReason": "end"}});
    assert!(dispatch_response(&msg, &pending, Some(&cleanup)).await);
    assert!(rx.await.unwrap().unwrap()["stopReason"] == "end");
    assert!(!busy.load(Ordering::SeqCst));
    assert_eq!(prompt_rpc_id.load(Ordering::SeqCst), 0);
    assert!(trace_writer.lock().await.is_none());
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
