pub(crate) struct ReaderLoopFinishCtx<'a> {
    pub trace_opts: ReaderTraceLineOpts,
    pub verbose_coalesce: &'a mut VerboseIoCoalescer,
    pub trace_coalesce: &'a mut TraceChunkCoalescer,
    pub trace_writer: &'a Arc<Mutex<Option<PromptTraceWriter>>>,
    pub reader_dead: &'a Arc<AtomicBool>,
    pub pending: &'a Arc<Mutex<HashMap<u64, ResponseTx>>>,
}

pub(crate) async fn reader_loop_finish(session: &mut ReaderLoopFinishCtx<'_>) {
    if session.trace_opts.acp_verbose {
        session.verbose_coalesce.flush_all();
    }
    flush_trace_coalesce(session).await;
    reader_dead_after_stdout_close(session).await;
}

pub(crate) async fn flush_trace_coalesce(session: &mut ReaderLoopFinishCtx<'_>) {
    let mut g = session.trace_writer.lock().await;
    if let Some(ref mut f) = *g {
        for (kind, tl, stream, upgrade_plan) in session.trace_coalesce.flush_all() {
            crate::acp::trace_file_write_line(
                f,
                &tl,
                Some(kind),
                crate::acp::trace_line_write::TraceFileStdout {
                    tee_stdout: session.trace_opts.tee_trace_stdout,
                    stream_iterable_closed: stream,
                    stream_upgrade_plan: upgrade_plan,
                    tee_line_override: None,
                    tee_line_display: None,
                    ts: None,
                },
            )
            .await;
        }
    }
}

pub(crate) async fn reader_dead_after_stdout_close(session: &mut ReaderLoopFinishCtx<'_>) {
    session.reader_dead.store(true, Ordering::SeqCst);
    let closed_msg = "acp stdout closed".to_string();
    let mut g = session.pending.lock().await;
    for (_, tx) in g.drain() {
        let _ = tx.send(Err(closed_msg.clone()));
    }
}

pub(crate) struct ReaderLoopLineIo<'a> {
    pub trace_opts: &'a ReaderTraceLineOpts,
    pub trace_writer: &'a Arc<Mutex<Option<PromptTraceWriter>>>,
    pub coalescers: &'a mut VerboseTraceCoalesceState<'a>,
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
        trace_jsonl,
        prompt_round_health,
    } = inp;
    let trace_opts = ReaderTraceLineOpts {
        acp_verbose,
        tee_trace_stdout,
    };
    let mut verbose_coalesce = VerboseIoCoalescer::default();
    let mut trace_coalesce = TraceChunkCoalescer::default();
    reader_loop_drain_stdout(ReaderLoopDrainCtx {
        stdout,
        trace_opts: &trace_opts,
        trace_writer: &trace_writer,
        verbose_coalesce: &mut verbose_coalesce,
        trace_coalesce: &mut trace_coalesce,
        pending: &pending,
        stdin: &stdin,
        acp_activity_seq: &acp_activity_seq,
        acp_activity_notify: &acp_activity_notify,
        prompt_cleanup: prompt_cleanup.as_deref(),
        acp_verbose,
        trace_jsonl: trace_jsonl.as_deref(),
        prompt_round_health: &prompt_round_health,
    })
    .await;
    reader_loop_finish(&mut ReaderLoopFinishCtx {
        trace_opts,
        verbose_coalesce: &mut verbose_coalesce,
        trace_coalesce: &mut trace_coalesce,
        trace_writer: &trace_writer,
        reader_dead: &reader_dead,
        pending: &pending,
    })
    .await;
}

pub(crate) struct ReaderLoopDrainCtx<'a> {
    stdout: tokio::process::ChildStdout,
    trace_opts: &'a ReaderTraceLineOpts,
    trace_writer: &'a Arc<Mutex<Option<PromptTraceWriter>>>,
    verbose_coalesce: &'a mut VerboseIoCoalescer,
    trace_coalesce: &'a mut TraceChunkCoalescer,
    pending: &'a Arc<Mutex<HashMap<u64, ResponseTx>>>,
    stdin: &'a Arc<Mutex<ChildStdin>>,
    acp_activity_seq: &'a Arc<AtomicU64>,
    acp_activity_notify: &'a Arc<tokio::sync::Notify>,
    prompt_cleanup: Option<&'a PromptRpcCleanup>,
    acp_verbose: bool,
    trace_jsonl: Option<&'a AcpJsonlTrace>,
    prompt_round_health: &'a Arc<std::sync::Mutex<PromptRoundHealth>>,
}

pub(crate) async fn reader_loop_drain_stdout(ctx: ReaderLoopDrainCtx<'_>) {
    let ReaderLoopDrainCtx {
        stdout,
        trace_opts,
        trace_writer,
        verbose_coalesce,
        trace_coalesce,
        pending,
        stdin,
        acp_activity_seq,
        acp_activity_notify,
        prompt_cleanup,
        acp_verbose,
        trace_jsonl,
        prompt_round_health,
    } = ctx;
    let mut lines = BufReader::new(stdout).lines();
    let mut line_io = ReaderLoopLineIo {
        trace_opts,
        trace_writer,
        coalescers: &mut VerboseTraceCoalesceState {
            verbose: verbose_coalesce,
            trace: trace_coalesce,
        },
    };
    while let Ok(Some(line)) = lines.next_line().await {
        reader_loop_on_line(
            &line,
            &mut line_io,
            IncomingLineDispatch {
                pending,
                stdin,
                acp_activity_seq,
                acp_activity_notify,
                prompt_cleanup,
                acp_verbose,
                trace_jsonl,
                prompt_round_health,
            },
        )
        .await;
    }
}

pub(crate) async fn reader_loop_on_line(
    line: &str,
    io: &mut ReaderLoopLineIo<'_>,
    dispatch: IncomingLineDispatch<'_>,
) {
    reader_loop_verbose_and_trace_line(line, io.trace_opts, io.trace_writer, io.coalescers).await;
    handle_incoming_line(line, dispatch).await;
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
            trace_jsonl: args.trace_jsonl,
            prompt_round_health: args.prompt_round_health,
        })
        .await;
    })
}

#[cfg(test)]
#[path = "reader_stdout_body_b_kiss_cov_test.rs"]
mod reader_stdout_body_b_kiss_cov_test;
#[cfg(test)]
#[path = "reader_stdout_body_b_kiss_test.rs"]
mod reader_stdout_body_b_kiss_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<ReaderLoopDrainCtx> = None;
        let _: Option<ReaderLoopFinishCtx> = None;
        let _: Option<ReaderLoopLineIo> = None;
    }
}
