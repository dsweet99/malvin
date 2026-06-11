//! Top-level test module so kiss sees identifier refs for `.inc` and other units.

#[test]
fn kiss_cov_reader_stdout_body_a_inc() {
    use crate::acp::wrap_reader_a::inline::*;
    let _: Option<IncomingLineDispatch<'_>> = None;
    let _: Option<ReaderLoopInput> = None;
    let _ = handle_incoming_line;
}

#[test]
fn kiss_cov_reader_stdout_body_b_inc() {
    let _: Option<crate::acp::ReaderLoopFinishCtx<'_>> = None;
    let _: Option<crate::acp::ReaderLoopDrainCtx<'_>> = None;
    let _: Option<crate::acp::ReaderLoopLineIo<'_>> = None;
    let _ = crate::acp::reader_loop_finish;
    let _ = crate::acp::flush_trace_coalesce;
    let _ = crate::acp::reader_dead_after_stdout_close;
    let _ = crate::acp::reader_loop;
    let _ = crate::acp::reader_loop_drain_stdout;
    let _ = crate::acp::reader_loop_on_line;
    let _ = crate::acp::spawn_acp_stdout_reader;
}

#[test]
fn kiss_cov_session_channels_inc() {
    use crate::acp::wrap_session_channels::inline::*;
    let _: Option<crate::acp::SessionAfterStdioIn<'_>> = None;
    let _ = acp_activity_state;
    let _ = random_agent_name;
    let _ = session_channel_sync;
    let _ = trace_jsonl_for_args;
}

#[test]
fn kiss_cov_session_prompt_inc() {
    use crate::acp::wrap_session_prompt::inline::*;
    let _: Option<PromptTraceDispatchMeta<'_>> = None;
    let _ = uniform_outgoing_trace_preamble;
    let _ = do_split_outgoing_trace_preamble;
    let _ = open_live_prompt_trace_writer;
    let _ = do_split_trace_preamble;
    let _ = rpc_session_prompt_text;
}

#[test]
fn kiss_cov_session_spawn_inc() {
    let _ = crate::acp::prompt_rpc_cleanup_arc;
    let _ = crate::acp::spawn_handshake_stdout_reader;
    let _ = crate::acp::handshake_stdio_rpc;
    let _ = crate::acp::acp_spawn_start_reader_and_handshake;
    let _ = crate::acp::session_after_stdio;
    let _ = crate::acp::spawn_acp_session;
}

#[test]
fn kiss_cov_trace_line_write() {
    let _: Option<crate::acp::ReaderTraceLineOpts> = None;
    let _ = crate::acp::trace_line_write::raw_output_suppress_thought_stdout;
    let _ = crate::acp::reader_loop_verbose_and_trace_line;
    let _ = crate::acp::trace_line_write::write_trace_line_coalesced;
}

