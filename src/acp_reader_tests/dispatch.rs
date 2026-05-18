use super::acp_activity_state;
use crate::acp::ResponseTx;
use crate::acp::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::process::Command;
use tokio::sync::oneshot;
use tokio::sync::{Mutex, Notify};

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
include!("dispatch_unix.rs");

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

