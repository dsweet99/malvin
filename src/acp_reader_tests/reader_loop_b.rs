use super::spawn_reader_true_stdout_pending_eof;
use crate::acp::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::oneshot;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_reader_loop_drains_pending_on_stdout_eof() {
    let mut h = spawn_reader_true_stdout_pending_eof(
        crate::acp_memory_containment::AcpMemoryContainment::inactive(),
    )
    .await;
    let err = h.rx.await.unwrap().unwrap_err();
    assert!(err.contains("closed") || err.contains("acp"));
    let _: () = h.waiter.await.unwrap();
    let _ = h.stdout_child.wait().await;
    let _ = h.stdin_holder.kill().await;
}

#[cfg(unix)]
mod reader_loop_eof_unix {
    use super::spawn_reader_true_stdout_pending_eof;
    use crate::acp_memory_containment::{AGENT_EXCEEDED_MEMORY_LIMIT_MSG, test_support};

    #[tokio::test]
    async fn test_reader_loop_maps_memory_limit_on_stdout_eof() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
        let memory_containment = test_support::active_with_cgroup_dir(dir.path().to_path_buf());
        std::fs::write(dir.path().join("memory.events"), "oom_kill 1\n").expect("events");
        let mut h = spawn_reader_true_stdout_pending_eof(memory_containment).await;
        let err = h.rx.await.unwrap().unwrap_err();
        assert_eq!(err, AGENT_EXCEEDED_MEMORY_LIMIT_MSG);
        let _: () = h.waiter.await.unwrap();
        let _ = h.stdout_child.wait().await;
        let _ = h.stdin_holder.kill().await;
    }
}

#[cfg(unix)]
pub(super) use reader_loop_eof_unix::*;

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
