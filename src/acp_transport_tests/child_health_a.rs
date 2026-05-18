
async fn spawn_json_activity_then_response(
    seq: Arc<AtomicU64>,
    notify: Arc<Notify>,
    tx: tokio::sync::oneshot::Sender<Result<Value, String>>,
) {
    tokio::time::sleep(Duration::from_millis(20)).await;
    note_acp_json_activity(&seq, &notify);
    tokio::time::sleep(Duration::from_millis(20)).await;
    note_acp_json_activity(&seq, &notify);
    tokio::time::sleep(Duration::from_millis(20)).await;
    let _ = tx.send(Ok(json!({"ok": true})));
}

async fn spawn_activity_then_kill_child(
    seq: Arc<AtomicU64>,
    notify: Arc<Notify>,
    kill_pid: Option<u32>,
) {
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    note_acp_json_activity(&seq, &notify);
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
    if let Some(pid) = kill_pid {
        let _ = std::process::Command::new("kill")
            .arg("-KILL")
            .arg(pid.to_string())
            .status();
    }
}

#[tokio::test]
async fn rpc_request_with_correlation_id_stays_alive_while_json_updates_arrive() {
    let request_id = 3u64;
    let h = RpcSleepHarness::spawn_sleep("5", SleepStdoutDrainMode::SmallBuf).await;
    let (pending_tx, _pending_rx) = tokio::sync::oneshot::channel::<Result<Value, String>>();
    h.pending.lock().await.insert(request_id, pending_tx);
    let (tx, rx) = tokio::sync::oneshot::channel();
    let seq = h.acp_activity_seq.clone();
    let notify = h.acp_activity_notify.clone();
    tokio::spawn(spawn_json_activity_then_response(seq, notify, tx));

    let start = tokio::time::Instant::now();
    let res = harness_rpc_wait(HarnessRpcWaitParams {
        h: &h,
        request_id,
        timeout: Duration::from_millis(25),
        rx,
        child_pid: None,
    })
        .await
        .expect("ACP activity should extend the timeout window");
    assert_eq!(res["ok"], true);
    assert!(
        start.elapsed() >= Duration::from_millis(60),
        "response arrived before timeout extensions should have run: {:?}",
        start.elapsed()
    );
    assert!(
        !h.pending.lock().await.contains_key(&request_id),
        "request should be removed from pending after completion"
    );
    h.shutdown().await;
}

#[tokio::test]
async fn rpc_wait_response_reports_dead_child_after_silence() {
    let h = RpcSleepHarness::spawn_sleep("10", SleepStdoutDrainMode::SmallBuf).await;
    let child_pid = h.child_pid();
    let (_tx, rx) = tokio::sync::oneshot::channel::<Result<Value, String>>();
    let seq = h.acp_activity_seq.clone();
    let notify = h.acp_activity_notify.clone();
    tokio::spawn(spawn_activity_then_kill_child(seq, notify, child_pid));

    let err = tokio::time::timeout(
        std::time::Duration::from_millis(220),
        harness_rpc_wait(HarnessRpcWaitParams {
            h: &h,
            request_id: 7,
            timeout: std::time::Duration::from_millis(25),
            rx,
            child_pid,
        }),
    )
    .await
    .expect("timed out waiting for request completion")
    .expect_err("expected child-health timeout");
    assert!(
        err.contains("acp child process is not running")
            || err.contains("acp child process is zombie"),
        "{err}"
    );
    h.shutdown().await;
}
