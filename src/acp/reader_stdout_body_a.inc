// ACP stdout read loop and JSON-RPC line dispatch.

pub(crate) struct ReaderSpawnArgs {
    pub stdout: ChildStdout,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<Notify>,
    pub reader_dead: Arc<AtomicBool>,
    pub trace_writer: Arc<Mutex<Option<PromptTraceWriter>>>,
    pub prompt_cleanup: Arc<PromptRpcCleanup>,
    pub acp_verbose: bool,
    pub tee_trace_stdout: bool,
    pub trace_jsonl: Option<Arc<AcpJsonlTrace>>,
    pub prompt_round_health: Arc<std::sync::Mutex<PromptRoundHealth>>,
}

pub(crate) struct ReaderLoopInput {
    pub stdout: ChildStdout,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<Notify>,
    pub reader_dead: Arc<AtomicBool>,
    pub trace_writer: Arc<Mutex<Option<PromptTraceWriter>>>,
    pub prompt_cleanup: Option<Arc<PromptRpcCleanup>>,
    pub acp_verbose: bool,
    pub tee_trace_stdout: bool,
    pub trace_jsonl: Option<Arc<AcpJsonlTrace>>,
    pub prompt_round_health: Arc<std::sync::Mutex<PromptRoundHealth>>,
}

/// JSON-RPC 2.0 allows `id` as string or number; map nonnegative integers (and decimal strings) to
/// `u64` for pending lookup.
/// Correlation id for `session/request_permission`: JSON-RPC root `id`, or `params.id` /
/// `params.requestId` when the server nests it (some peers omit the top-level field).
pub(crate) fn request_permission_correlation_id(msg: &Value) -> Option<&Value> {
    if let Some(id) = msg.get("id")
        && !id.is_null()
    {
        return Some(id);
    }
    let params = msg.get("params")?;
    let obj = params.as_object()?;
    if let Some(id) = obj.get("id")
        && !id.is_null()
    {
        return Some(id);
    }
    let id = obj.get("requestId")?;
    if id.is_null() {
        None
    } else {
        Some(id)
    }
}

pub(crate) fn jsonrpc_response_id_as_u64(id_v: &Value) -> Option<u64> {
    if let Some(n) = id_v.as_u64() {
        return Some(n);
    }
    if let Some(n) = id_v.as_i64()
        && n >= 0
    {
        return u64::try_from(n).ok();
    }
    id_v.as_str()?.parse::<u64>().ok()
}

pub(crate) async fn dispatch_response(
    msg: &Value,
    pending: &Arc<Mutex<HashMap<u64, ResponseTx>>>,
    prompt_cleanup: Option<&PromptRpcCleanup>,
) -> bool {
    let Some(id_v) = msg.get("id") else {
        return false;
    };
    let Some(id) = jsonrpc_response_id_as_u64(id_v) else {
        warn!(id = ?id_v, "acp response id is not a nonnegative integer or decimal string");
        return false;
    };
    let Some(tx) = pending.lock().await.remove(&id) else {
        debug!(id, "acp response for unknown request");
        return true;
    };
    if let Some(c) = prompt_cleanup {
        c.clear_if_prompt_response(id).await;
    }
    if let Some(err) = msg.get("error") {
        let _ = tx.send(Err(crate::acp::format_jsonrpc_error(err)));
    } else if let Some(res) = msg.get("result") {
        let _ = tx.send(Ok(res.clone()));
    } else {
        let _ = tx.send(Err("acp response missing result/error".into()));
    }
    true
}

pub(crate) struct IncomingLineDispatch<'a> {
    pub pending: &'a Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub stdin: &'a Arc<Mutex<ChildStdin>>,
    pub sandbox_guest_in: Option<&'a tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    pub acp_activity_seq: &'a Arc<AtomicU64>,
    pub acp_activity_notify: &'a Arc<Notify>,
    pub prompt_cleanup: Option<&'a PromptRpcCleanup>,
    pub acp_verbose: bool,
    pub trace_jsonl: Option<&'a AcpJsonlTrace>,
    pub prompt_round_health: &'a Arc<std::sync::Mutex<PromptRoundHealth>>,
}

pub(crate) async fn handle_incoming_line(line: &str, d: IncomingLineDispatch<'_>) {
    if let Some(trace) = d.trace_jsonl {
        trace.append_line("in", line);
    }
    note_acp_trace_activity(d.acp_activity_seq, d.acp_activity_notify);
    let msg: Value = match serde_json::from_str(line) {
        Ok(m) => m,
        Err(e) => {
            warn!(error = %e, "acp stdout JSON parse error");
            return;
        }
    };

    match msg.get("method").and_then(|m| m.as_str()) {
        None => {
            let _ = dispatch_response(&msg, d.pending, d.prompt_cleanup).await;
        }
        Some("session/update") => {
            d.prompt_round_health
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .record_session_update(&msg);
            trace!(target: "malvin::acp", update = %msg, "session/update");
        }
        Some("session/request_permission") => {
            let Some(id) = request_permission_correlation_id(&msg) else {
                warn!(
                    target: "malvin::acp",
                    "session/request_permission missing correlation id (top-level or params.id/requestId); cannot reply"
                );
                return;
            };
            let body = json!({
                "jsonrpc": "2.0",
                "id": id.clone(),
                "result": {
                    "outcome": { "outcome": "selected", "optionId": "allow-always" }
                }
            });
            let line = match serde_json::to_string(&body) {
                Ok(l) => l,
                Err(e) => {
                    error!(error = %e, "failed to answer session/request_permission");
                    return;
                }
            };
            if let Err(e) = write_rpc_line(
                d.stdin,
                d.sandbox_guest_in,
                RpcLineWriteOpts {
                    line: &line,
                    acp_verbose: d.acp_verbose,
                    trace_jsonl: d.trace_jsonl,
                    activity: Some((d.acp_activity_seq, d.acp_activity_notify)),
                },
            )
            .await
            {
                error!(error = %e, "failed to answer session/request_permission");
            }
        }
        Some(method) => {
            trace!(target: "malvin::acp", method, "acp notification or server request (ignored)");
        }
    }
}

