// `initialize` / `authenticate` / `session/new` handshake.

pub(crate) struct HandshakeParams<'a> {
    pub io: &'a AcpStdioRpc,
    pub next_id: &'a Arc<AtomicU64>,
    pub cwd: &'a Path,
    pub rpc_timeout: std::time::Duration,
    pub require_cursor_login_auth: bool,
    pub child_pid: Option<u32>,
}

pub(crate) async fn handshake_inner(p: HandshakeParams<'_>) -> Result<String, String> {
    let init = json!({
        "protocolVersion": 1,
        "clientCapabilities": {
            "fs": { "readTextFile": false, "writeTextFile": false },
            "terminal": false
        },
        "clientInfo": { "name": "malvin", "version": env!("CARGO_PKG_VERSION") }
    });
    let _ = rpc_request(RpcRequestNext {
        io: p.io,
        next_id: p.next_id,
        method: "initialize",
        params: init,
        rpc_timeout: p.rpc_timeout,
        child_pid: p.child_pid,
    })
    .await
    .map_err(|e| format!("ACP `initialize` failed: {e}"))?;
    if p.require_cursor_login_auth {
        let _ = rpc_request(RpcRequestNext {
            io: p.io,
            next_id: p.next_id,
            method: "authenticate",
            params: json!({ "methodId": "cursor_login" }),
            rpc_timeout: p.rpc_timeout,
            child_pid: p.child_pid,
        })
        .await
        .map_err(|e| format!("ACP `authenticate` (methodId=cursor_login) failed: {e}"))?;
    }
    let new_params = json!({
        "cwd": p.cwd.to_string_lossy(),
        "mcpServers": []
    });
    let res = rpc_request(RpcRequestNext {
        io: p.io,
        next_id: p.next_id,
        method: "session/new",
        params: new_params,
        rpc_timeout: p.rpc_timeout,
        child_pid: p.child_pid,
    })
        .await
        .map_err(|e| format!("ACP `session/new` failed: {e}"))?;
    let sid = res
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "session/new missing sessionId".to_string())?;
    Ok(sid.to_string())
}

#[test]
fn kiss_stringify_handshake() {
    let _ = stringify!(HandshakeParams);
    let _ = stringify!(handshake_inner);
}

