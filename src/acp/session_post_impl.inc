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
        let mut ch = self.0.child.lock().await;
        match ch.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(e) => {
                warn!(error = %e, "child try_wait failed; assuming still alive");
                true
            }
        }
    }

    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.0.busy.load(Ordering::SeqCst)
    }

    async fn send_rpc(&self, method: &str, params: Value) -> Result<Value, String> {
        let io = acp_stdio(&self.0);
        let child_pid = self.0.child.lock().await.id();
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
        trace_prepare_file(trace_path).await?;
        let mut file = trace_open_truncated(trace_path).await?;
        let meta = match &trace {
            OutgoingPromptTrace::Uniform(u) => {
                uniform_outgoing_trace_preamble(self, &mut file, text, u).await?
            }
            OutgoingPromptTrace::DoSplit(split) => {
                do_split_outgoing_trace_preamble(self, &mut file, split).await?
            }
        };
        *self.0.trace_writer.lock().await = Some(PromptTraceWriter::for_live_prompt(
            LivePromptTraceArgs {
                file,
                who: meta.incoming_tag,
                plain_lines: meta.plain_lines,
                stdout_replacement: prompt_stdout_replacement(meta.stdout_replacement_who),
                trace_raw_output: meta.trace_raw_output,
                show_thoughts_on_stdout: self.0.show_thoughts_on_stdout,
                emit_stdout_markdown: self.0.emit_stdout_markdown
                    && !meta.trace_raw_output
                    && !meta.plain_lines,
                work_dir: self.0.work_dir.clone(),
                run_timing: self.0.run_timing.clone(),
                session_id: self.0.session_id.clone(),
            },
        ));
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
            Ok(()) => Ok(()),
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
        self.reset_prompt_inflight().await;
        unix_process_group::terminate_process_group(self.0.process_group_id).await;
        let mut ch = self.0.child.lock().await;
        let _ = ch.kill().await;
        let wait_result = ch
            .wait()
            .await
            .map_err(|e| format!("acp wait: {e}"));
        drop(ch);
        wait_result?;
        Ok(())
    }

    /// Snapshot of ACP tool failures observed during the most recent [`Self::prompt`].
    #[must_use]
    pub fn take_prompt_round_health(&self) -> PromptRoundHealth {
        std::mem::take(
            &mut self
                .0
                .prompt_round_health
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }
}

impl Drop for AcpSession {
    fn drop(&mut self) {}
}


