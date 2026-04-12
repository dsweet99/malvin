// JSON-RPC request/response over ACP stdio.

/// Shared stdio transport state for JSON-RPC to `agent acp`.
pub(crate) struct AcpStdioRpc {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<tokio::sync::Notify>,
    pub acp_verbose: bool,
}

pub(crate) async fn write_rpc_line(
    stdin: &Arc<Mutex<ChildStdin>>,
    line: &str,
    acp_verbose: bool,
) -> Result<(), String> {
    if acp_verbose {
        info!(
            target: "malvin::acp::io",
            direction = "out",
            line = %line,
            "acp message"
        );
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
    Ok(())
}

/// One outbound JSON-RPC request (correlation id chosen by caller).
pub(crate) struct RpcOutgoing<'a> {
    pub io: &'a AcpStdioRpc,
    pub id: u64,
    pub method: &'a str,
    pub params: Value,
    pub rpc_timeout: std::time::Duration,
}

/// Next-id JSON-RPC request (`id` from [`AtomicU64`]).
pub(crate) struct RpcRequestNext<'a> {
    pub io: &'a AcpStdioRpc,
    pub next_id: &'a Arc<AtomicU64>,
    pub method: &'a str,
    pub params: Value,
    pub rpc_timeout: std::time::Duration,
}

pub(crate) struct RpcWaitArgs<'a> {
    pub pending: &'a Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: &'a Arc<AtomicU64>,
    pub acp_activity_notify: &'a Arc<tokio::sync::Notify>,
    pub id: u64,
    pub rpc_timeout: std::time::Duration,
    pub rx: oneshot::Receiver<Result<Value, String>>,
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
    if let Err(e) = write_rpc_line(&io.stdin, &line, io.acp_verbose).await {
        io.pending.lock().await.remove(&o.id);
        return Err(e);
    }
    rpc_wait_response(RpcWaitArgs {
        pending: &io.pending,
        acp_activity_seq: &io.acp_activity_seq,
        acp_activity_notify: &io.acp_activity_notify,
        id: o.id,
        rpc_timeout: o.rpc_timeout,
        rx,
    })
    .await
}

pub(crate) async fn rpc_request(n: RpcRequestNext<'_>) -> Result<Value, String> {
    let id = n
        .next_id
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    rpc_request_with_correlation_id(RpcOutgoing {
        io: n.io,
        id,
        method: n.method,
        params: n.params,
        rpc_timeout: n.rpc_timeout,
    })
    .await
}

pub(crate) async fn rpc_wait_response(args: RpcWaitArgs<'_>) -> Result<Value, String> {
    let mut rx = args.rx;
    let mut seen_activity = args
        .acp_activity_seq
        .load(std::sync::atomic::Ordering::SeqCst);
    loop {
        let activity = args.acp_activity_notify.notified();
        tokio::pin!(activity);
        let latest = args
            .acp_activity_seq
            .load(std::sync::atomic::Ordering::SeqCst);
        if latest != seen_activity {
            seen_activity = latest;
            continue;
        }
        let timeout = tokio::time::sleep(args.rpc_timeout);
        tokio::pin!(timeout);
        tokio::select! {
            ready_recv = &mut rx => {
                return ready_recv
                    .map_err(|_| "acp request canceled (session dropped)".to_string())?;
            }
            () = &mut activity => {
                seen_activity = args
                    .acp_activity_seq
                    .load(std::sync::atomic::Ordering::SeqCst);
            }
            () = &mut timeout => {
                args.pending.lock().await.remove(&args.id);
                return Err("acp RPC timed out".into());
            }
        }
    }
}

#[test]
fn kiss_stringify_rpc_a() {
    let _ = stringify!(AcpStdioRpc);
    let _ = stringify!(write_rpc_line);
    let _ = stringify!(RpcOutgoing);
    let _ = stringify!(RpcRequestNext);
    let _ = stringify!(RpcWaitArgs);
}

#[test]
fn kiss_stringify_rpc_b() {
    let _ = stringify!(rpc_request_with_correlation_id);
    let _ = stringify!(rpc_request);
    let _ = stringify!(rpc_wait_response);
}
