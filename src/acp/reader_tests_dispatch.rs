use crate::acp::ResponseTx;
use crate::acp::*;
use serde_json::json;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::oneshot;

#[cfg(unix)]
use crate::acp_test_unix_bin::unix_bin_with_fallback;

use super::reader_tests_helpers::acp_activity_state;

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
mod incoming_line_unix {
    use super::*;

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
                trace_jsonl: None,
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
                trace_jsonl: None,
            },
        )
        .await;
        assert_eq!(
            acp_activity_seq.load(Ordering::SeqCst),
            2,
            "each received stdout line counts as trace activity"
        );
    }
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
