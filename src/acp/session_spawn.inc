// Spawn `agent acp`, stdout reader, and JSON-RPC handshake (inlined into `acp` for `kiss` dependency depth).

fn prompt_rpc_cleanup_arc(io: &AcpHandshakeIo) -> Arc<PromptRpcCleanup> {
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
    memory_containment: &crate::acp_memory_containment::AcpMemoryContainment,
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
        memory_containment: memory_containment.clone(),
    });
}

fn handshake_stdio_rpc(
    io: &AcpHandshakeIo,
    session: &AcpHandshakeSessionOpts,
    memory_containment: &crate::acp_memory_containment::AcpMemoryContainment,
) -> AcpStdioRpc {
    AcpStdioRpc {
        reader_dead: io.reader_dead.clone(),
        stdin: io.stdin.clone(),
        pending: io.pending.clone(),
        acp_activity_seq: io.acp_activity_seq.clone(),
        acp_activity_notify: io.acp_activity_notify.clone(),
        acp_verbose: session.acp_verbose,
        trace_jsonl: io.trace_jsonl.clone(),
        memory_containment: memory_containment.clone(),
    }
}

async fn kill_child_and_finalize_containment(
    child: &mut Child,
    memory_containment: &crate::acp_memory_containment::AcpMemoryContainment,
) {
    let _ = child.kill().await;
    let _ = child.wait().await;
    crate::acp_memory_containment::finalize_containment_cgroup(memory_containment);
}

pub(crate) async fn acp_spawn_start_reader_and_handshake(
    pipes: AcpChildStdout,
    io: AcpHandshakeIo,
    cont: AcpHandshakeContinuation<'_>,
    memory_containment: crate::acp_memory_containment::AcpMemoryContainment,
) -> Result<(Child, String), String> {
    let mut child = pipes.child;
    let stdout = pipes.stdout;
    let child_pid = child.id();
    spawn_handshake_stdout_reader(stdout, &io, &cont.session, &memory_containment);

    let io_rpc = handshake_stdio_rpc(&io, &cont.session, &memory_containment);
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
            kill_child_and_finalize_containment(&mut child, &memory_containment).await;
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
        memory_containment,
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
        memory_containment.clone(),
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
            memory_containment,
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
    let containment_handle =
        crate::acp_memory_containment::begin_containment_for_command(&mut cmd);
    #[cfg(target_os = "linux")]
    let had_cgroup_plan = matches!(
        &containment_handle,
        crate::acp_memory_containment::ContainmentHandle::Linux { .. }
    );
    let mut child = match spawn_agent_acp_child(&mut cmd).await {
        Ok(child) => child,
        Err(e) => {
            crate::acp_memory_containment::remove_containment_handle(containment_handle);
            return Err(format!("failed to spawn agent acp: {e}"));
        }
    };
    #[cfg(target_os = "linux")]
    let memory_containment =
        crate::acp_memory_containment::complete_and_require_linux_containment_after_spawn(
            child.id(),
            containment_handle,
            had_cgroup_plan,
            &mut child,
        )
        .await?;
    #[cfg(not(target_os = "linux"))]
    let memory_containment = crate::acp_memory_containment::complete_containment_after_spawn(
        child.id(),
        containment_handle,
    )
    .await;
    crate::acp_memory_containment::emit_containment_unavailable_warn_after_spawn(
        crate::acp_memory_containment::ContainmentUnavailableWarnAtSpawn {
            log_full_outgoing_prompts: args.log_full_outgoing_prompts,
            containment_active: memory_containment.active(),
        },
    );
    let (stdin, stdout) = match take_stdio_pipes(&mut child).await {
        Ok(pipes) => pipes,
        Err(e) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            crate::acp_memory_containment::finalize_containment_cgroup(&memory_containment);
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
        memory_containment,
    })
    .await
}

