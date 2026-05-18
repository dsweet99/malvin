
#[tokio::test]
async fn rpc_response_arriving_during_child_health_grace_is_delivered() {
    let h = RpcSleepHarness::spawn_sleep("10", SleepStdoutDrainMode::SmallBuf).await;
    let child_pid = h.child_pid();
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<Value, String>>();
    let (pending_tx, _pending_rx) = tokio::sync::oneshot::channel::<Result<Value, String>>();
    let request_id = 99_u64;
    h.pending.lock().await.insert(request_id, pending_tx);
    let timeout = Duration::from_millis(100);
    let grace = crate::child_health::silence_grace_for_rpc_timeout(timeout);
    let response_time = timeout + grace / 2;
    tokio::spawn({
        let tx = tx;
        async move {
            tokio::time::sleep(response_time).await;
            let _ = tx.send(Ok(json!({"ok": true})));
        }
    });
    let start = tokio::time::Instant::now();
    let res = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        harness_rpc_wait(HarnessRpcWaitParams {
            h: &h,
            request_id,
            timeout,
            rx,
            child_pid,
        }),
    )
    .await
    .expect("request wait timed out")
    .expect("response should arrive before health grace expires");
    assert!(start.elapsed() >= response_time);
    assert_eq!(res["ok"], true);
    assert!(
        !h.pending.lock().await.contains_key(&request_id),
        "request should be removed from pending after completion"
    );
    h.shutdown().await;
}

#[test]
fn active_memory_containment_maps_timeout_message_when_oom() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
    let memory_containment = crate::acp_memory_containment::test_support::active_with_cgroup_dir(
        dir.path().to_path_buf(),
    );
    std::fs::write(dir.path().join("memory.events"), "oom_kill 1\n").expect("events");
    assert!(memory_containment.memory_limit_exceeded());
    assert_eq!(
        crate::acp_memory_containment::map_acp_child_exit_message(
            &memory_containment,
            "acp request id 1 timed out",
        ),
        crate::acp_memory_containment::AGENT_EXCEEDED_MEMORY_LIMIT_MSG
    );
}
