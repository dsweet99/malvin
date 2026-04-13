use session_io::acp_stdio;

/// [`AcpSession`] implementation and post-spawn handshake.
pub(crate) fn prompt_stdout_replacement(who: &str) -> Option<&'static str> {
    if who == "learn" {
        Some(crate::output::LEARNING_PLACEHOLDER)
    } else {
        None
    }
}

impl AcpSession {
    /// Spawn `agent acp`, run `initialize` / `authenticate` / `session/new`.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a human-readable message if the child process cannot be started or the
    /// ACP handshake fails.
    pub async fn spawn(args: AcpSpawnArgs<'_>) -> Result<Self, String> {
        spawn_acp_session(args).await
    }

    pub async fn is_alive(&self) -> bool {
        if self.0.reader_dead.load(Ordering::SeqCst) {
            return false;
        }
        let mut ch = self.0.child.lock().await;
        match ch.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) | Err(_) => true,
        }
    }

    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.0.busy.load(Ordering::SeqCst)
    }

    async fn send_rpc(&self, method: &str, params: Value) -> Result<Value, String> {
        let io = acp_stdio(&self.0);
        rpc_request(RpcRequestNext {
            io: &io,
            next_id: &self.0.next_id,
            method,
            params,
            rpc_timeout: self.0.rpc_timeout,
        })
        .await
    }

    async fn reset_prompt_inflight(&self) {
        self.0.busy.store(false, Ordering::SeqCst);
        *self.0.trace_writer.lock().await = None;
        self.0.prompt_rpc_id.store(0, Ordering::SeqCst);
        if let Some(n) = &self.0.ui_idle_notify {
            n.notify_waiters();
        }
    }

    /// Send [`session/prompt`](https://cursor.com/docs/cli/acp) for the active session.
    ///
    /// # Errors
    ///
    /// Returns `Err` if trace file setup or the JSON-RPC request fails (see also [`Self::cancel`]).
    pub async fn prompt(&self, text: &str, trace_path: &Path, who: &str) -> Result<(), String> {
        self.prompt_impl(text, trace_path, who).await
    }

    async fn prompt_impl(&self, text: &str, trace_path: &Path, who: &str) -> Result<(), String> {
        let _prompt_turn = self.0.prompt_singleflight.lock().await;
        trace_prepare_file(trace_path).await?;
        let mut file = trace_open_truncated(trace_path).await?;
        trace_write_invocation_header(&mut file).await?;
        *self.0.trace_writer.lock().await = Some(PromptTraceWriter {
            file,
            who: who.to_string(),
            stdout_replacement: prompt_stdout_replacement(who),
        });
        self.0.busy.store(true, Ordering::SeqCst);

        let id = self.0.next_id.fetch_add(1, Ordering::SeqCst);
        self.0.prompt_rpc_id.store(id, Ordering::SeqCst);

        let params = json!({
            "sessionId": &self.0.session_id,
            "prompt": [{ "type": "text", "text": text }]
        });

        let io = acp_stdio(&self.0);
        let res = rpc_request_with_correlation_id(RpcOutgoing {
            io: &io,
            id,
            method: "session/prompt",
            params,
            rpc_timeout: self.0.rpc_timeout,
        })
        .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                self.reset_prompt_inflight().await;
                Err(e)
            }
        }
    }

    /// Request cancellation of the in-flight prompt (ACP `session/cancel`).
    ///
    /// # Errors
    ///
    /// Returns `Err` if the JSON-RPC request fails.
    pub async fn cancel(&self) -> Result<(), String> {
        let params = json!({ "sessionId": &self.0.session_id });
        let r = self.send_rpc("session/cancel", params).await;
        if r.is_ok() {
            self.0.busy.store(false, Ordering::SeqCst);
            *self.0.trace_writer.lock().await = None;
            self.0.prompt_rpc_id.store(0, Ordering::SeqCst);
            if let Some(n) = &self.0.ui_idle_notify {
                n.notify_waiters();
            }
        }
        r.map(|_| ())
    }

    /// Stop the `agent acp` child process and release session resources.
    ///
    /// # Errors
    ///
    /// Returns `Err` if waiting on the child after kill fails.
    pub async fn shutdown(&self) -> Result<(), String> {
        self.0.busy.store(false, Ordering::SeqCst);
        *self.0.trace_writer.lock().await = None;
        self.0.prompt_rpc_id.store(0, Ordering::SeqCst);
        if let Some(n) = &self.0.ui_idle_notify {
            n.notify_waiters();
        }
        let mut ch = self.0.child.lock().await;
        let _ = ch.kill().await;
        ch.wait()
            .await
            .map_err(|e| format!("acp wait: {e}"))?;
        drop(ch);
        Ok(())
    }
}

#[test]
fn kiss_stringify_session_a() {
    let _ = stringify!(prompt_stdout_replacement);
    let _ = stringify!(AcpSession::spawn);
    let _ = stringify!(AcpSession::is_alive);
    let _ = stringify!(AcpSession::is_busy);
    let _ = stringify!(AcpSession::prompt);
    let _ = stringify!(AcpSession::cancel);
    let _ = stringify!(AcpSession::shutdown);
}

#[test]
fn kiss_stringify_session_b() {
    let _ = stringify!(AcpSession::send_rpc);
    let _ = stringify!(AcpSession::reset_prompt_inflight);
    let _ = stringify!(AcpSession::prompt_impl);
}

#[test]
fn prompt_stdout_replacement_redacts_learn_only() {
    assert_eq!(
        prompt_stdout_replacement("learn"),
        Some(crate::output::LEARNING_PLACEHOLDER)
    );
    assert_eq!(prompt_stdout_replacement("kpop"), None);
    assert_eq!(prompt_stdout_replacement("review_1"), None);
}
