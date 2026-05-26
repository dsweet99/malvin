// Spawn `agent acp`, stdout reader, and JSON-RPC handshake (inlined into `acp` for `kiss` dependency depth).

pub(crate) fn prompt_rpc_cleanup_arc(io: &AcpHandshakeIo) -> Arc<PromptRpcCleanup> {
    Arc::new(PromptRpcCleanup {
        busy: io.busy.clone(),
        trace_writer: io.trace_writer.clone(),
        prompt_rpc_id: io.prompt_rpc_id.clone(),
        idle_notify: io.ui_idle_notify.clone(),
    })
}

fn spawn_handshake_stdout_reader(
    stdout: ChildStdout,
    io: &AcpHandshakeIo,
    session: &AcpHandshakeSessionOpts,
) {
    let prompt_cleanup = prompt_rpc_cleanup_arc(io);
    let _reader_task = spawn_acp_stdout_reader(ReaderSpawnArgs {
        stdout,
        pending: io.pending.clone(),
        stdin: io.stdin.clone(),
        acp_activity_seq: io.acp_activity_seq.clone(),
        acp_activity_notify: io.acp_activity_notify.clone(),
        reader_dead: io.reader_dead.clone(),
        trace_writer: io.trace_writer.clone(),
        prompt_cleanup,
        acp_verbose: session.acp_verbose,
        tee_trace_stdout: session.tee_trace_stdout,
        trace_jsonl: io.trace_jsonl.clone(),
        prompt_round_health: io.prompt_round_health.clone(),
    });
}

fn handshake_stdio_rpc(io: &AcpHandshakeIo, session: &AcpHandshakeSessionOpts) -> AcpStdioRpc {
    AcpStdioRpc {
        reader_dead: io.reader_dead.clone(),
        stdin: io.stdin.clone(),
        pending: io.pending.clone(),
        acp_activity_seq: io.acp_activity_seq.clone(),
        acp_activity_notify: io.acp_activity_notify.clone(),
        acp_verbose: session.acp_verbose,
        trace_jsonl: io.trace_jsonl.clone(),
    }
}

pub(crate) async fn acp_spawn_start_reader_and_handshake(
    pipes: AcpChildStdout,
    io: AcpHandshakeIo,
    cont: AcpHandshakeContinuation<'_>,
) -> Result<(Child, String), String> {
    let mut child = pipes.child;
    let stdout = pipes.stdout;
    let child_pid = child.id();
    spawn_handshake_stdout_reader(stdout, &io, &cont.session);

    let io_rpc = handshake_stdio_rpc(&io, &cont.session);
    match handshake_inner(HandshakeParams {
        io: &io_rpc,
        next_id: &io.next_id,
        cwd: cont.cwd,
        rpc_timeout: cont.rpc_timeout,
        require_cursor_login_auth: cont.session.require_cursor_login_auth,
        child_pid,
    })
    .await
    {
        Ok(session_id) => Ok((child, session_id)),
        Err(e) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            Err(e)
        }
    }
}

pub(crate) async fn session_after_stdio(inp: SessionAfterStdioIn<'_>) -> Result<AcpSession, String> {
    let SessionAfterStdioIn {
        args,
        rpc_timeout,
        require_cursor_login_auth,
        child,
        stdin,
        stdout,
    } = inp;
    let process_group_id = child.id();
    let ch = SessionChannelState::new(stdin, &args);
    let hs_io = ch.handshake_io();
    let (child, session_id) = acp_spawn_start_reader_and_handshake(
        AcpChildStdout { child, stdout },
        hs_io,
        AcpHandshakeContinuation {
            cwd: args.cwd,
            rpc_timeout,
            session: AcpHandshakeSessionOpts {
                acp_verbose: args.acp_verbose,
                require_cursor_login_auth,
                tee_trace_stdout: args.tee_trace_stdout,
            },
        },
    )
    .await?;
    let trace_jsonl = ch.trace_jsonl.clone();
    Ok(AcpSession(Arc::new(ch.into_session_inner(
        SessionInnerAssembly {
            child,
            process_group_id,
            session_id,
            rpc_timeout,
            telemetry: SessionReaderTelemetry {
                acp_verbose: args.acp_verbose,
                raw_output: args.raw_output,
                show_thoughts_on_stdout: args.show_thoughts_on_stdout,
                emit_stdout_markdown: args.emit_stdout_markdown,
                prompts_log_run_dir: args.prompts_log_run_dir.map(std::path::Path::to_path_buf),
                log_full_outgoing_prompts: args.log_full_outgoing_prompts,
                trace_jsonl,
            },
            work_dir: args.cwd.to_path_buf(),
            run_timing: None,
        },
    ))))
}

pub(crate) async fn spawn_acp_session(args: AcpSpawnArgs<'_>) -> Result<AcpSession, String> {
    let rpc_timeout = clamp_rpc_timeout(args.rpc_timeout);
    let require_cursor_login_auth =
        requires_cursor_login_auth(args.api_key, args.auth_token);
    let mut cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: args.cwd,
        bin_override: args.bin_override,
        api_key: args.api_key,
        auth_token: args.auth_token,
        george_acp_lane: args.george_acp_lane,
        model: args.model,
        force: args.force,
    });
    let mut child = spawn_agent_acp_child(&mut cmd)
        .await
        .map_err(|e| format!("failed to spawn agent acp: {e}"))?;
    let (stdin, stdout) = match take_stdio_pipes(&mut child).await {
        Ok(pipes) => pipes,
        Err(e) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(e);
        }
    };
    session_after_stdio(SessionAfterStdioIn {
        args,
        rpc_timeout,
        require_cursor_login_auth,
        child,
        stdin,
        stdout,
    })
    .await
}
