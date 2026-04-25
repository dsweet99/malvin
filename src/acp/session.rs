use session_io::acp_stdio;

use outgoing_prompt_trace::{
    DoPromptTraceSplit, OutgoingPromptTrace, UniformOutgoingTrace,
};

/// [`AcpSession`] implementation and post-spawn handshake.
pub(crate) fn prompt_stdout_replacement(who: &str) -> Option<&'static str> {
    if who == "learn" {
        Some(crate::output::LEARNING_PLACEHOLDER)
    } else {
        None
    }
}

async fn rpc_session_prompt_text(session: &AcpSession, text: &str, id: u64) -> Result<(), String> {
    let params = json!({
        "sessionId": &session.0.session_id,
        "prompt": [{ "type": "text", "text": text }]
    });
    let io = acp_stdio(&session.0);
    rpc_request_with_correlation_id(RpcOutgoing {
        io: &io,
        id,
        method: "session/prompt",
        params,
        rpc_timeout: session.0.rpc_timeout,
    })
    .await
    .map(|_| ())
}

const fn is_prompt_payload_trailing_ws(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r'
}

fn trim_prompt_payload_trailing_ws(s: &str) -> &str {
    s.trim_end_matches(is_prompt_payload_trailing_ws)
}

async fn do_split_trace_preamble(
    file: &mut tokio::fs::File,
    raw_output: bool,
    split: &DoPromptTraceSplit<'_>,
) -> Result<(String, &'static str, bool, bool), String> {
    trace_write_invocation_and_do_split_prompt(file, split).await?;
    if !raw_output {
        crate::output::print_outgoing_prompt_log("do");
    }
    Ok((
        crate::output::format_acp_directional_tag_prefix('<', "do"),
        "do",
        raw_output,
        true,
    ))
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
            style_text: split.style_text,
            header_text: split.header,
            user_text: split.user,
        });
        let normalized_text = trim_prompt_payload_trailing_ws(text);
        let normalized_expected = trim_prompt_payload_trailing_ws(&expected);
        if normalized_text != normalized_expected {
            return Err("prompt_do_trace_split: text does not match split parts".to_string());
        }
        self.prompt_impl(&expected, trace_path, OutgoingPromptTrace::DoSplit(split))
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
        let (incoming_tag, stdout_replacement_who, trace_raw_output, plain_lines) = match &trace {
            OutgoingPromptTrace::Uniform(u) => {
                trace_write_invocation_header(&mut file).await?;
                trace_write_outgoing_prompt(&mut file, u.trace_who, text).await?;
                if !self.0.raw_output {
                    let outgoing_label = u.stdout_bracket_label.unwrap_or(u.trace_who);
                    crate::output::print_outgoing_prompt_log(outgoing_label);
                }
                (
                    crate::output::format_acp_directional_tag_prefix('<', u.trace_who),
                    u.trace_who,
                    self.0.raw_output,
                    false,
                )
            }
            OutgoingPromptTrace::DoSplit(split) => {
                do_split_trace_preamble(&mut file, self.0.raw_output, split).await?
            }
        };
        *self.0.trace_writer.lock().await = Some(PromptTraceWriter {
            file,
            who: incoming_tag,
            plain_lines,
            stdout_replacement: prompt_stdout_replacement(stdout_replacement_who),
            placeholder_emitted: false,
            raw_output: trace_raw_output,
            emit_stdout_markdown: self.0.emit_stdout_markdown
                && !trace_raw_output
                && !plain_lines,
        });
        self.0.busy.store(true, Ordering::SeqCst);

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
        if r.is_ok() {
            self.reset_prompt_inflight().await;
        }
        r.map(|_| ())
    }

    /// Stop the `agent acp` child process and release session resources.
    ///
    /// # Errors
    ///
    /// Returns `Err` if waiting on the child after kill fails.
    pub async fn shutdown(&self) -> Result<(), String> {
        self.reset_prompt_inflight().await;
        let mut ch = self.0.child.lock().await;
        let _ = ch.kill().await;
        ch.wait()
            .await
            .map_err(|e| format!("acp wait: {e}"))?;
        drop(ch);
        Ok(())
    }
}
