use crate::acp::*;
use crate::acp::ResponseTx;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::process::Command;
use tokio::sync::oneshot;
use tokio::sync::{Mutex, Notify};

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
    crate::acp::trace_line_write::write_trace_line_coalesced(
        &mut writer,
        &mut c,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
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
    crate::acp::trace_line_write::write_trace_line_coalesced(
        &mut writer,
        &mut c,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
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

