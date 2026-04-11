// JSON-RPC request/response over ACP stdio.

/// Shared stdio transport state for JSON-RPC to `agent acp`.
pub(crate) struct AcpStdioRpc {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
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
    rpc_wait_response(&io.pending, o.id, o.rpc_timeout, rx).await
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

pub(crate) async fn rpc_wait_response(
    pending: &Arc<Mutex<HashMap<u64, ResponseTx>>>,
    id: u64,
    rpc_timeout: std::time::Duration,
    rx: oneshot::Receiver<Result<Value, String>>,
) -> Result<Value, String> {
    if let Ok(ready_recv) = tokio::time::timeout(rpc_timeout, rx).await {
        ready_recv.map_err(|_| "acp request canceled (session dropped)".to_string())?
    } else {
        pending.lock().await.remove(&id);
        Err("acp RPC timed out".into())
    }
}

#[test]
fn kiss_stringify_rpc_a() {
    let _ = stringify!(AcpStdioRpc);
    let _ = stringify!(write_rpc_line);
    let _ = stringify!(RpcOutgoing);
    let _ = stringify!(RpcRequestNext);
}

#[test]
fn kiss_stringify_rpc_b() {
    let _ = stringify!(rpc_request_with_correlation_id);
    let _ = stringify!(rpc_request);
    let _ = stringify!(rpc_wait_response);
}
