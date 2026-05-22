// JSON-RPC request/response over ACP stdio.

/// Shared stdio transport state for JSON-RPC to `agent acp`.
pub(crate) struct AcpStdioRpc {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<tokio::sync::Notify>,
    pub acp_verbose: bool,
    pub trace_jsonl: Option<Arc<AcpJsonlTrace>>,
    pub memory_containment: crate::acp_memory_containment::AcpMemoryContainment,
}

pub(crate) async fn write_rpc_line(
    stdin: &Arc<Mutex<ChildStdin>>,
    line: &str,
    acp_verbose: bool,
    trace_jsonl: Option<&AcpJsonlTrace>,
    activity: Option<(&Arc<AtomicU64>, &Arc<tokio::sync::Notify>)>,
) -> Result<(), String> {
    if acp_verbose {
        info!(
            target: "malvin::acp::io",
            direction = "out",
            line = %line,
            "acp message"
        );
    }
    if let Some(trace) = trace_jsonl {
        trace.append_line("out", line);
    }
    let mut guard = stdin.lock().await;
    guard
        .write_all(line.as_bytes())
        .await
        .map_err(|e| format!("acp stdin write: {e}"))?;
    guard
        .write_all(b"\n")
        .await
        .map_err(|e| format!("acp stdin newline: {e}"))?;
    guard
        .flush()
        .await
        .map_err(|e| format!("acp stdin flush: {e}"))?;
    drop(guard);
    if let Some((seq, notify)) = activity {
        note_acp_trace_activity(seq, notify);
    }
    Ok(())
}

/// One outbound JSON-RPC request (correlation id chosen by caller).
#[allow(dead_code)]
pub(crate) struct RpcOutgoing<'a> {
    pub io: &'a AcpStdioRpc,
    pub id: u64,
    pub method: &'a str,
    pub params: Value,
    pub rpc_timeout: std::time::Duration,
    pub child_pid: Option<u32>,
}

/// Next-id JSON-RPC request (`id` from [`AtomicU64`]).
#[allow(dead_code)]
pub(crate) struct RpcRequestNext<'a> {
    pub io: &'a AcpStdioRpc,
    pub next_id: &'a Arc<AtomicU64>,
    pub method: &'a str,
    pub params: Value,
    pub rpc_timeout: std::time::Duration,
    pub child_pid: Option<u32>,
}

#[allow(dead_code)]
pub(crate) struct RpcWaitArgs<'a> {
    pub _pending: &'a Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: &'a Arc<AtomicU64>,
    pub acp_activity_notify: &'a Arc<tokio::sync::Notify>,
    pub _id: u64,
    pub rx: oneshot::Receiver<Result<Value, String>>,
    pub child_pid: Option<u32>,
}

pub(crate) async fn rpc_request_with_correlation_id(o: RpcOutgoing<'_>) -> Result<Value, String> {
    let io = o.io;
    if io.reader_dead.load(std::sync::atomic::Ordering::SeqCst) {
        return Err("acp session is dead".into());
    }
    let (tx, rx) = oneshot::channel();
    io.pending.lock().await.insert(o.id, tx);
    let req = json!({
        "jsonrpc": "2.0",
        "id": o.id,
        "method": o.method,
        "params": o.params
    });
    let line = match serde_json::to_string(&req) {
        Ok(l) => l,
        Err(e) => {
            io.pending.lock().await.remove(&o.id);
            return Err(e.to_string());
        }
    };
    if let Err(e) = write_rpc_line(
        &io.stdin,
        &line,
        io.acp_verbose,
        io.trace_jsonl.as_deref(),
        Some((&io.acp_activity_seq, &io.acp_activity_notify)),
    )
    .await
    {
        io.pending.lock().await.remove(&o.id);
        return Err(e);
    }
    let args = RpcWaitArgs {
        _pending: &io.pending,
        acp_activity_seq: &io.acp_activity_seq,
        acp_activity_notify: &io.acp_activity_notify,
        _id: o.id,
        rx,
        child_pid: o.child_pid,
    };
    let wait_state = (
        &io.acp_activity_seq,
        &io.acp_activity_notify,
        &io.pending,
        o.child_pid,
        &io.memory_containment,
    );
    rpc_wait_with_timeout(
        o.id,
        o.rpc_timeout,
        rpc_wait_response(args),
        wait_state,
    )
    .await
}

pub(crate) async fn rpc_wait_with_timeout(
    id: u64,
    timeout: std::time::Duration,
    wait: impl std::future::Future<Output = Result<Value, String>>,
    state: RpcWaitContext<'_>,
) -> Result<Value, String> {
    let (acp_activity_seq, acp_activity_notify, pending, child_pid, memory_containment) = state;
    tokio::pin!(wait);
    loop {
        tokio::select! {
            biased;
            ready_recv = &mut wait => {
                let result = ready_recv?;
                pending.lock().await.remove(&id);
                return Ok(result);
            }
            () = acp_activity_notify.notified() => {
                let _ = acp_activity_seq
                    .load(std::sync::atomic::Ordering::SeqCst);
            }
            () = tokio::time::sleep(timeout) => {
                let timeout_err = if let Some(child_pid) = child_pid {
                    let grace = crate::child_health::silence_grace_for_rpc_timeout(timeout);
                    let health = crate::child_health::evaluate_after_acp_silence(child_pid, grace);
                    tokio::pin!(health);
                    tokio::select! {
                        biased;
                        ready_recv = &mut wait => {
                            let result = ready_recv?;
                            pending.lock().await.remove(&id);
                            return Ok(result);
                        }
                        outcome = &mut health => {
                            match outcome {
                                crate::child_health::SilenceHealthOutcome::StillBusyExtendWait => {
                                    Ok(())
                                }
                                crate::child_health::SilenceHealthOutcome::AppearsHung => {
                                    let err = child_health_timeout_error(
                                        crate::child_health::SilenceHealthOutcome::AppearsHung,
                                        memory_containment,
                                    )
                                    .unwrap_or_else(|| {
                                        format!("acp request id {id} timed out after {timeout:?}")
                                    });
                                    Err(err)
                                }
                                other => {
                                    let err = child_health_timeout_error(other, memory_containment)
                                    .unwrap_or_else(|| {
                                        format!("acp request id {id} timed out after {timeout:?}")
                                    });
                                    Err(err)
                                }
                            }
                        }
                    }
                } else {
                    Err(format!("acp request id {id} timed out after {timeout:?}"))
                };
                if timeout_err.is_ok() {
                    continue;
                }
                pending.lock().await.remove(&id);
                return Err(timeout_err.expect_err("timeout outcome must be Err"));
            }
        }
    }
}
