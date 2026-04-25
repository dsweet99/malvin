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

pub(crate) fn note_acp_json_activity(
    acp_activity_seq: &Arc<AtomicU64>,
    acp_activity_notify: &Arc<Notify>,
) {
    acp_activity_seq.fetch_add(1, Ordering::SeqCst);
    acp_activity_notify.notify_waiters();
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
    pub acp_activity_seq: &'a Arc<AtomicU64>,
    pub acp_activity_notify: &'a Arc<Notify>,
    pub prompt_cleanup: Option<&'a PromptRpcCleanup>,
    pub acp_verbose: bool,
}

pub(crate) async fn handle_incoming_line(line: &str, d: IncomingLineDispatch<'_>) {
    let msg: Value = match serde_json::from_str(line) {
        Ok(m) => m,
        Err(e) => {
            warn!(error = %e, "acp stdout JSON parse error");
            return;
        }
    };
    note_acp_json_activity(d.acp_activity_seq, d.acp_activity_notify);
    match msg.get("method").and_then(|m| m.as_str()) {
        None => {
            let _ = dispatch_response(&msg, d.pending, d.prompt_cleanup).await;
        }
        Some("session/update") => {
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
            if let Err(e) = write_rpc_line(d.stdin, &line, d.acp_verbose).await {
                error!(error = %e, "failed to answer session/request_permission");
            }
        }
        Some(method) => {
            trace!(target: "malvin::acp", method, "acp notification or server request (ignored)");
        }
    }
}

pub(crate) async fn reader_loop(inp: ReaderLoopInput) {
    let ReaderLoopInput {
        stdout,
        pending,
        stdin,
        acp_activity_seq,
        acp_activity_notify,
        reader_dead,
        trace_writer,
        prompt_cleanup,
        acp_verbose,
        tee_trace_stdout,
    } = inp;
    let trace_opts = ReaderTraceLineOpts {
        acp_verbose,
        tee_trace_stdout,
    };
    let mut lines = BufReader::new(stdout).lines();
    let mut verbose_coalesce = VerboseIoCoalescer::default();
    let mut trace_coalesce = TraceChunkCoalescer::default();
    let mut coalescers = VerboseTraceCoalesceState {
        verbose: &mut verbose_coalesce,
        trace: &mut trace_coalesce,
    };
    while let Ok(Some(line)) = lines.next_line().await {
        reader_loop_verbose_and_trace_line(
            &line,
            &trace_opts,
            &trace_writer,
            &mut coalescers,
        )
        .await;
        let pc = prompt_cleanup.as_deref();
        handle_incoming_line(
            &line,
            IncomingLineDispatch {
                pending: &pending,
                stdin: &stdin,
                acp_activity_seq: &acp_activity_seq,
                acp_activity_notify: &acp_activity_notify,
                prompt_cleanup: pc,
                acp_verbose,
            },
        )
        .await;
    }
    if acp_verbose {
        verbose_coalesce.flush_all();
    }
    {
        let mut g = trace_writer.lock().await;
        if let Some(ref mut f) = *g {
            for (kind, tl) in trace_coalesce.flush_all() {
                crate::acp::trace_file_write_line(f, &tl, tee_trace_stdout, Some(kind)).await;
            }
        }
    }
    reader_dead.store(true, Ordering::SeqCst);
    let mut g = pending.lock().await;
    for (_, tx) in g.drain() {
        let _ = tx.send(Err("acp stdout closed".into()));
    }
}

pub(crate) fn spawn_acp_stdout_reader(args: ReaderSpawnArgs) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        reader_loop(ReaderLoopInput {
            stdout: args.stdout,
            pending: args.pending,
            stdin: args.stdin,
            acp_activity_seq: args.acp_activity_seq,
            acp_activity_notify: args.acp_activity_notify,
            reader_dead: args.reader_dead,
            trace_writer: args.trace_writer,
            prompt_cleanup: Some(args.prompt_cleanup),
            acp_verbose: args.acp_verbose,
            tee_trace_stdout: args.tee_trace_stdout,
        })
        .await;
    })
}
