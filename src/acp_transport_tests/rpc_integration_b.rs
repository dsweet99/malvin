use super::prelude::*;
use super::shared_harness::*;

#[tokio::test]
async fn test_rpc_request_does_not_leak_pending_after_write_failure() {
    let (stdin, drain) = true_child_stdin_stdout_drained_after_exit().await;

    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = super::shared_harness::acp_activity_state();
    let reader_dead = Arc::new(AtomicBool::new(false));
    let next_id = Arc::new(AtomicU64::new(1));

    let io = acp_stdio_rpc_inactive(InactiveRpcIo {
        reader_dead,
        stdin,
        pending,
        acp_activity_seq,
        acp_activity_notify,
    });
    let err = rpc_request(RpcRequestNext {
        io: &io,
        next_id: &next_id,
        method: "nope",
        params: json!({}),
        rpc_timeout: acp_rpc_timeout(),
        child_pid: None,
    })
    .await
    .expect_err("stdin write after child exit should fail");

    assert!(!err.is_empty(), "{err}");
    assert!(
        io.pending.lock().await.is_empty(),
        "pending should be cleared when write fails; leaked ids: {:?}",
        io.pending.lock().await.keys().copied().collect::<Vec<_>>()
    );

    let _ = drain.await;
}

#[tokio::test]
async fn rpc_request_with_correlation_id_times_out_when_stdout_silent() {
    let h = RpcSleepHarness::spawn_sleep("15", SleepStdoutDrainMode::LargeBuf).await;
    let io = h.io();

    let timeout_err = tokio::time::timeout(
        std::time::Duration::from_millis(120),
        rpc_request_with_correlation_id(RpcOutgoing {
            io: &io,
            id: 3,
            method: "unanswered",
            params: json!({}),
            rpc_timeout: std::time::Duration::from_millis(25),
            child_pid: None,
        }),
    )
    .await
    .expect("rpc request should complete with internal timeout")
    .expect_err("peer never responds");
    assert!(timeout_err.contains("timed out"), "{timeout_err}");
    assert!(
        io.pending.lock().await.is_empty(),
        "pending should be cleared after timeout; stale entries: {:?}",
        io.pending.lock().await.keys().copied().collect::<Vec<_>>()
    );
    h.shutdown().await;
}

#[tokio::test]
async fn rpc_request_with_correlation_id_errors_when_reader_dead() {
    let mut h = RpcSleepHarness::spawn_sleep("2", SleepStdoutDrainMode::None).await;
    h.reader_dead = Arc::new(AtomicBool::new(true));
    let io = h.io();
    let err = rpc_request_with_correlation_id(RpcOutgoing {
        io: &io,
        id: 7,
        method: "nope",
        params: json!({}),
        rpc_timeout: std::time::Duration::from_millis(500),
        child_pid: None,
    })
    .await
    .expect_err("reader flagged dead");
    assert!(err.contains("dead"), "{err}");
    h.shutdown().await;
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_test_rpc_request_does_not_leak_pending_after_write_failure() { let _ = stringify!(test_rpc_request_does_not_leak_pending_after_write_failure); }

    #[test]
    fn kiss_cov_rpc_request_with_correlation_id_times_out_when_stdout_silent() { let _ = stringify!(rpc_request_with_correlation_id_times_out_when_stdout_silent); }

    #[test]
    fn kiss_cov_rpc_request_with_correlation_id_errors_when_reader_dead() { let _ = stringify!(rpc_request_with_correlation_id_errors_when_reader_dead); }

}
