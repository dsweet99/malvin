// Spawn `agent acp`, stdout reader, and JSON-RPC handshake (inlined into `acp` for `kiss` dependency depth).
use handshake_types::{
    AcpChildStdout, AcpHandshakeContinuation, AcpHandshakeIo, AcpHandshakeSessionOpts,
};
use session_channels::{SessionAfterStdioIn, SessionChannelState};
use session_io::{clamp_rpc_timeout, take_stdio_pipes};

pub(crate) async fn acp_spawn_start_reader_and_handshake(
    pipes: AcpChildStdout,
    io: AcpHandshakeIo,
    cont: AcpHandshakeContinuation<'_>,
) -> Result<(Child, String), String> {
    let mut child = pipes.child;
    let stdout = pipes.stdout;
    let prompt_cleanup = Arc::new(PromptRpcCleanup {
        busy: io.busy.clone(),
        trace_writer: io.trace_writer.clone(),
        prompt_rpc_id: io.prompt_rpc_id.clone(),
        idle_notify: io.ui_idle_notify.clone(),
    });
    let _reader_task = spawn_acp_stdout_reader(ReaderSpawnArgs {
        stdout,
        pending: io.pending.clone(),
        stdin: io.stdin.clone(),
        reader_dead: io.reader_dead.clone(),
        trace_writer: io.trace_writer.clone(),
        prompt_cleanup,
        acp_verbose: cont.session.acp_verbose,
        tee_trace_stdout: cont.session.tee_trace_stdout,
    });

    let io_rpc = AcpStdioRpc {
        reader_dead: io.reader_dead.clone(),
        stdin: io.stdin.clone(),
        pending: io.pending.clone(),
        acp_verbose: cont.session.acp_verbose,
    };
    let session_id = match handshake_inner(HandshakeParams {
        io: &io_rpc,
        next_id: &io.next_id,
        cwd: cont.cwd,
        rpc_timeout: cont.rpc_timeout,
        require_cursor_login_auth: cont.session.require_cursor_login_auth,
    })
    .await
    {
        Ok(s) => s,
        Err(e) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(e);
        }
    };
    Ok((child, session_id))
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
    Ok(AcpSession(Arc::new(ch.into_session_inner(
        child,
        session_id,
        rpc_timeout,
        args.acp_verbose,
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
    let (stdin, stdout) = take_stdio_pipes(&mut child).await?;
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
