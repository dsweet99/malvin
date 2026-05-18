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

