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
    crate::acp::trace_line_write::write_trace_line_coalesced(
        &mut writer,
        &mut c,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
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

