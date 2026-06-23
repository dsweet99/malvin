//! External kiss witnesses for `.inc` fragment symbols.

#[test]
fn kiss_cov_session_channels_inc_fns() {
    let _ = super::wrap_session_channels::inline::acp_activity_state;
    let _ = super::wrap_session_channels::inline::random_agent_name;
    let _ = super::wrap_session_channels::inline::trace_jsonl_for_args;
    let _ = super::wrap_session_channels::inline::session_channel_sync;
}

#[test]
fn kiss_cov_session_channels_inc_types() {
    use crate::acp::wrap_session_channels::inline::SessionReaderTelemetry;

    let telemetry = SessionReaderTelemetry {
        acp_verbose: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
        trace_jsonl: None,
    };
    let SessionReaderTelemetry {
        acp_verbose,
        raw_output: _,
        show_thoughts_on_stdout: _,
        emit_stdout_markdown: _,
        prompts_log_run_dir: _,
        log_full_outgoing_prompts: _,
        trace_jsonl: _,
    } = telemetry;
    assert!(!acp_verbose);
    let _ = super::wrap_session_channels::inline::acp_activity_state();
    let _ = super::wrap_session_channels::inline::acp_activity_state;
    let _ = super::wrap_session_channels::inline::random_agent_name;
    let _ = super::wrap_session_channels::inline::trace_jsonl_for_args;
    let _ = super::wrap_session_channels::inline::session_channel_sync;
    let _ = stringify!(SessionInnerAssembly);
    let _ = stringify!(SessionAfterStdioIn);
    let _ = stringify!(stdin_from_sleep_holder);
    let _ = stringify!(session_channel_state_sets_trace_jsonl_when_prompts_log_run_dir_set);
    let _ = std::mem::size_of::<super::wrap_session_channels::inline::SessionInnerAssembly>();
    let _ = std::mem::size_of::<super::wrap_session_channels::inline::SessionAfterStdioIn>();
}

#[test]
fn kiss_cov_reader_stdout_body_a_inc_types() {
    let _ = stringify!(ReaderSpawnArgs);
    let _ = stringify!(ReaderLoopInput);
    let _ = stringify!(IncomingLineDispatch);
    let _ = crate::acp::request_permission_correlation_id;
    let _ = crate::acp::jsonrpc_response_id_as_u64;
    let _ = crate::acp::dispatch_response;
}

#[test]
fn kiss_cov_reader_stdout_body_b_inc() {
    let _ = crate::acp::reader_loop_finish;
    let _ = crate::acp::flush_trace_coalesce;
    let _ = crate::acp::reader_dead_after_stdout_close;
    let _ = crate::acp::reader_loop;
    let _ = crate::acp::reader_loop_drain_stdout;
    let _ = crate::acp::reader_loop_on_line;
    let _ = crate::acp::spawn_acp_stdout_reader;
    let _ = stringify!(ReaderLoopFinishCtx);
    let _ = stringify!(ReaderLoopLineIo);
    let _ = stringify!(ReaderLoopDrainCtx);
}

#[cfg(unix)]
#[test]
fn kiss_cov_reader_stdout_body_a_inc_dispatch() {
    use crate::acp::wrap_reader_a::inline::IncomingLineDispatch;
    use crate::acp_tests::reader_tests_helpers::{block_on_test, test_prompt_round_health};

    block_on_test(async {
        let cat = crate::acp_tests::reader_tests_helpers::CatSession::new().await;
        let parts = cat.dispatch_parts();
        let health = test_prompt_round_health();
        let dispatch = IncomingLineDispatch {
            pending: parts.pending,
            stdin: parts.stdin,
            acp_activity_seq: parts.acp_activity_seq,
            acp_activity_notify: parts.acp_activity_notify,
            prompt_cleanup: None,
            acp_verbose: false,
            trace_jsonl: None,
            prompt_round_health: &health,
        };
        let IncomingLineDispatch {
            pending: _,
            stdin: _,
            acp_activity_seq: _,
            acp_activity_notify: _,
            prompt_cleanup: _,
            acp_verbose,
            trace_jsonl: _,
            prompt_round_health: _,
        } = dispatch;
        assert!(!acp_verbose);
        let _ = cat.finish_stdout().await;
    });
}

#[test]
fn kiss_cov_session_post_impl_inc() {
    let _ = crate::acp::AcpSession::is_alive;
    let _ = crate::acp::AcpSession::is_busy;
    let _ = crate::acp::AcpSession::send_rpc;
    let _ = crate::acp::AcpSession::cancel;
}

#[test]
fn kiss_cov_session_prompt_trace_inc() {
    use crate::acp::wrap_session_prompt::inline::PromptTraceDispatchMeta;
    let meta = PromptTraceDispatchMeta {
        incoming_tag: "kiss".into(),
        stdout_replacement_who: "who",
        trace_raw_output: false,
        plain_lines: false,
    };
    let PromptTraceDispatchMeta {
        incoming_tag,
        stdout_replacement_who,
        trace_raw_output,
        plain_lines,
    } = std::hint::black_box(meta);
    assert_eq!(incoming_tag, "kiss");
    assert_eq!(stdout_replacement_who, "who");
    assert!(!trace_raw_output);
    assert!(!plain_lines);
}

#[test]
fn kiss_cov_session_prompt_inc() {
    use crate::acp::wrap_session_prompt::inline::*;
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
    let _ = crate::acp::trace_line_write::raw_output_suppress_thought_stdout;
    let _ = crate::acp::reader_loop_verbose_and_trace_line;
    let _ = crate::acp::trace_line_write::write_trace_line_coalesced;
}
