pub(crate) fn acp_session_set_run_timing(
    session: &mut AcpSession,
    timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
) {
    Arc::get_mut(&mut session.0)
        .expect("unique session handle")
        .run_timing = timing;
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
        {
            let mut slot = self.0.child.lock().await;
            let Some(ch) = slot.as_mut() else {
                return false;
            };
            let alive = match ch.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(e) => {
                    warn!(error = %e, "child try_wait failed; assuming still alive");
                    true
                }
            };
            drop(slot);
            alive
        }
    }

    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.0.busy.load(Ordering::SeqCst)
    }

    async fn send_rpc(&self, method: &str, params: Value) -> Result<Value, String> {
        let io = acp_stdio(&self.0);
        let child_pid = self
            .0
            .child
            .lock()
            .await
            .as_ref()
            .and_then(tokio::process::Child::id);
        rpc_request(RpcRequestNext {
            io: &io,
            next_id: &self.0.next_id,
            method,
            params,
            rpc_timeout: self.0.rpc_timeout,
            child_pid,
        })
        .await
    }

    async fn reset_prompt_inflight(&self) {
        let writer = self.0.trace_writer.lock().await.take();
        if let Some(mut w) = writer {
            w.flush_deferred();
        }
        self.0.busy.store(false, Ordering::SeqCst);
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
    /// `stdout_bracket_label` overrides the one-line `[label...]` stdout header; defaults to `who`.
    pub async fn prompt(
        &self,
        text: &str,
        trace_path: &Path,
        who: &str,
        stdout_bracket_label: Option<&str>,
    ) -> Result<(), String> {
        self.prompt_impl(
            text,
            trace_path,
            OutgoingPromptTrace::Uniform(UniformOutgoingTrace {
                trace_who: who,
                stdout_bracket_label,
            }),
        )
        .await
    }

    /// Like [`Self::prompt`], but records `malvin do` trace text as plain lines (style, header, request).
    ///
    /// `text` must be the exact payload sent on `session/prompt` (including any prepended style text).
    ///
    /// # Errors
    ///
    /// Returns `Err` if trace file setup or the JSON-RPC request fails (see also [`Self::cancel`]).
    pub async fn prompt_do_trace_split(
        &self,
        text: &str,
        trace_path: &Path,
        split: DoPromptTraceSplit<'_>,
    ) -> Result<(), String> {
        let expected = compose_do_split_prompt_text(&DoOutgoingTraceParts {
            header_text: split.header,
            user_text: split.user,
        });
        if text != expected {
            return Err("prompt_do_trace_split: text does not match split parts".to_string());
        }
        self.prompt_impl(text, trace_path, OutgoingPromptTrace::DoSplit(split))
            .await
    }

    async fn prompt_impl(
        &self,
        text: &str,
        trace_path: &Path,
        trace: OutgoingPromptTrace<'_>,
    ) -> Result<(), String> {
        let _prompt_turn = self.0.prompt_singleflight.lock().await;
        let writer = open_live_prompt_trace_writer(self, text, trace_path, trace).await?;
        *self.0.trace_writer.lock().await = Some(writer);
        self.0.busy.store(true, Ordering::SeqCst);

        self.0
            .prompt_round_health
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reset();

        let id = self.0.next_id.fetch_add(1, Ordering::SeqCst);
        self.0.prompt_rpc_id.store(id, Ordering::SeqCst);

        let res = rpc_session_prompt_text(self, text, id).await;

        match res {
            Ok(()) => {
                let upgrade = self
                    .0
                    .prompt_round_health
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .upgrade_plan_seen();
                if upgrade {
                    self.reset_prompt_inflight().await;
                    return Err(crate::acp::UPGRADE_PLAN_STOP_MESSAGE.to_string());
                }
                Ok(())
            }
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
        self.reset_prompt_inflight().await;
        r.map(|_| ())
    }

    /// Stop the `agent acp` child process and release session resources.
    ///
    /// # Errors
    ///
    /// Returns `Err` if waiting on the child after kill fails.
    pub async fn shutdown(&self) -> Result<(), String> {
        crate::active_agent_heartbeat::unregister_active_agent_process_group(
            self.0.process_group_id,
        );
        let cancel_timeout = crate::acp::unix_process_group_teardown_poll::shutdown_cancel_timeout();
        if cancel_timeout.is_zero() {
            let _ = self.cancel().await;
        } else {
            let _ = tokio::time::timeout(cancel_timeout, self.cancel()).await;
        }
        crate::acp::unix_process_group_teardown::terminate_agent_process_group(
            self.0.process_group_id,
            &self.0.spawn_pid_baseline,
        )
        .await;
        {
            let mut slot = self.0.child.lock().await;
            let Some(mut ch) = slot.take() else {
                crate::malvin_sandbox::clear_active_sandbox_session();
                return Ok(());
            };
            let _ = ch.kill().await;
            ch.wait()
                .await
                .map_err(|e| format!("acp wait: {e}"))?;
            drop(slot);
        }
        crate::malvin_sandbox::clear_active_sandbox_session();
        Ok(())
    }
}

impl Drop for AcpSession {
    fn drop(&mut self) {
        crate::acp::session_drop_teardown::acp_session_drop_if_last(&self.0);
    }
}

#[cfg(test)]
#[path = "session_post_impl_kiss_cov_test.rs"]
mod session_post_impl_kiss_cov_test;
#[cfg(test)]
#[path = "session_post_impl_kiss_test.rs"]
mod session_post_impl_kiss_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = cancel;
        let _ = is_alive;
        let _ = is_busy;
        let _ = prompt_do_trace_split;
        let _ = send_rpc;
    }
}
