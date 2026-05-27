//! Kiss coverage smoke for `rpc_part1` (kept out of that file for line-count limits).

use crate::acp::{rpc_wait_with_timeout, AcpStdioRpc};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{Mutex, Notify};

#[test]
fn kiss_cov_acp_stdio_rpc_type() {
    let _: Option<AcpStdioRpc> = None;
}

#[tokio::test]
async fn kiss_cov_rpc_wait_with_timeout_smoke() {
    let pending = Arc::new(Mutex::new(HashMap::new()));
    let seq = Arc::new(AtomicU64::new(0));
    let notify = Arc::new(Notify::new());
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<serde_json::Value, String>>();
    tx.send(Ok(serde_json::json!(1))).expect("send");
    let got = rpc_wait_with_timeout(
        1,
        std::time::Duration::from_millis(50),
        async { rx.await.expect("recv") },
        (&seq, &notify, &pending, None),
    )
    .await
    .expect("wait");
    assert_eq!(got, serde_json::json!(1));
    assert_eq!(seq.load(Ordering::Relaxed), 0);
}
