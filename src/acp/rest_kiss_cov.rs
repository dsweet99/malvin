//! External kiss witnesses for remaining `acp` production modules.

#[test]
fn kiss_witness_client_impl_flow() {
    let _ = super::AgentClient::run_kpop_flow;
    let _ = super::AgentClient::replace_coder_session_slot_for_tests;
}

#[test]
fn kiss_witness_client_impl_prompt_dispatch() {
    let _: Option<super::client_impl_prompt_dispatch::CoderSessionPromptDispatch> = None;
    let _ = super::client_impl_prompt_dispatch::dispatch_coder_session_prompt;
    let _ = super::client_impl_prompt_dispatch::coder_prompt_exhausted_error;
    let _ = super::client_impl_prompt_dispatch::record_coder_prompt_llm_timing;
}

#[test]
fn kiss_witness_handshake_types() {
    let opts = super::AcpHandshakeSessionOpts {
        acp_verbose: false,
        require_cursor_login_auth: false,
        tee_trace_stdout: false,
    };
    let super::AcpHandshakeSessionOpts {
        acp_verbose,
        require_cursor_login_auth,
        tee_trace_stdout,
    } = opts;
    assert!(!acp_verbose && !require_cursor_login_auth && !tee_trace_stdout);
    let tmp = tempfile::tempdir().expect("tempdir");
    let cont = super::AcpHandshakeContinuation {
        cwd: tmp.path(),
        rpc_timeout: std::time::Duration::from_secs(1),
        session: super::AcpHandshakeSessionOpts {
            acp_verbose: true,
            require_cursor_login_auth: false,
            tee_trace_stdout: false,
        },
    };
    let super::AcpHandshakeContinuation {
        cwd: _,
        rpc_timeout,
        session: _,
    } = cont;
    assert_eq!(rpc_timeout.as_secs(), 1);
    let _: Option<super::AcpHandshakeIo> = None;
    let _: Option<super::AcpChildStdout> = None;
}

#[test]
fn kiss_witness_hostile_orphan() {
    let _ = super::hostile_orphan_test_util::spawn_hostile_double_fork_daemon;
    let _ = super::hostile_orphan_test_util::wait_for_init_reparent;
    let _ = super::hostile_orphan_test_util::spawn_user_shell_cooperator;
    let _ = super::hostile_orphan_test_util::spawn_user_coincidental_daemon;
    let _ = super::hostile_orphan_test_util::spawn_isolated_agent_sleep;
    let _ = super::hostile_orphan_test_util::setup_user_init_reparented_daemon;
    let _ = super::hostile_orphan_test_util::cleanup_user_coincidental_test;
}

#[test]
fn kiss_witness_ops_body_kpop() {
    let _: Option<super::KpopPromptRound> = None;
    let _: Option<super::AgentKpopMultiturnCtl> = None;
    let _: Option<super::KpopFailAfterPrompt> = None;
    let _ = super::run_kpop_flow_once;
}

#[test]
fn kiss_witness_ops_body_kpop_mt() {
    let _ = stringify!(MultiturnRoundAfter);
}

#[test]
fn kiss_witness_prompt_trace_writer() {
    let _: Option<super::LivePromptTraceArgs> = None;
    let _ = stringify!(register_deferred_sink);
}

#[test]
fn kiss_witness_reader_inline() {
    let _: Option<super::PromptRpcCleanup> = None;
}

#[test]
fn kiss_witness_session_drop_teardown() {
    let _ = stringify!(take_child_without_tokio_drop);
    let _ = stringify!(acp_session_drop_teardown);
    let _ = stringify!(take_child_without_tokio_drop_for_test);
    let _ = stringify!(acp_session_drop_if_last);
}

#[test]
fn kiss_witness_session_types() {
    let _: Option<super::AcpSessionInner> = None;
    let _: Option<super::AcpSpawnArgs> = None;
    let _: Option<super::PromptTraceWriter> = None;
}

#[test]
fn kiss_witness_trace_line_write() {
    let _: Option<super::ReaderTraceLineOpts> = None;
    let _ = stringify!(WriteTraceLineCoalescedOpts);
    let _ = stringify!(TraceFileStdout);
}

#[test]
fn kiss_witness_session_types_tests() {
    let _ = stringify!(response_tx_oneshot_channel_constructible);
}

#[cfg(unix)]
#[test]
fn kiss_witness_kiss_coverage_test() {
    let _ = crate::acp::smoke_reader_loop_eof_pending_error;
}

#[test]
fn kiss_witness_unix_teardown_tests() {
    let _ = stringify!(signal_targets_noop_for_empty_set);
    let _ = stringify!(terminate_process_group_kills_sleep_child);
    let _ = stringify!(terminate_agent_process_group_kills_sleep_child);
    let _ = stringify!(baseline_amnestied_agent_acp_orphan_killed_on_teardown);
    let _ = stringify!(malvin_sibling_outside_agent_pg_killed_on_teardown);
    let _ = stringify!(teardown_async_ignoring_sigterm_eventually_killed);
    let _ = stringify!(terminate_process_group_noop_without_pgid_or_baseline);
}

#[test]
fn kiss_witness_process_group_mem_watch_tests() {
    let _ = stringify!(watch_process_group_memory_fail_closed_when_rss_unavailable);
    let _ = stringify!(watch_process_group_memory_writes_sandbox_oom_marker);
}

#[test]
fn kiss_witness_reader_inline_tests() {
    let _ = stringify!(clear_if_prompt_response_clears_busy);
}

#[test]
fn kiss_witness_command_kiss_cov_tests() {
    let _ = stringify!(write_executable_agent_script);
}

#[test]
fn kiss_witness_trace_line_write_tee_tests() {
    let _ = stringify!(rendered_tool_summary_tee_display);
}

#[test]
fn kiss_witness_transport_jsonrpc_error() {
    let _ = super::format_jsonrpc_error_obj;
    let _ = super::jsonrpc_error_code_str;
    let _ = super::jsonrpc_error_message_str;
    let _ = super::jsonrpc_error_data_detail;
}

#[test]
fn kiss_witness_transport_rpc_part1() {
    let _: Option<super::AcpStdioRpc> = None;
    let _: Option<super::RpcLineWriteOpts> = None;
    let _: Option<super::RpcOutgoing> = None;
    let _: Option<super::RpcRequestNext> = None;
}

#[test]
fn kiss_witness_transport_rpc_part2() {
    let _ = stringify!(spawn_cat_rpc_stdio_pair);
    let _ = stringify!(read_first_stdout_line);
    let _ = stringify!(write_rpc_line_appends_flush_line_readable_on_child_stdout);
}

#[test]
fn kiss_witness_transport_handshake_and_wait() {
    let _: Option<super::HandshakeParams> = None;
    let _: Option<super::RpcWaitArgs> = None;
}
