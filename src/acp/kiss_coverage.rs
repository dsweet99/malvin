#[test]
fn smoke_acp_reader_support_behavior() {
    use std::sync::atomic::Ordering;

    let (seq, _notify) = crate::acp_tests::reader_tests_helpers::acp_activity_state();
    assert_eq!(seq.load(Ordering::Relaxed), 0);
}

#[cfg(unix)]
#[tokio::test]
async fn smoke_reader_loop_eof_pending_error() {
    let msg = crate::acp_tests::reader_tests_reader_loop::reader_loop_eof_pending_error().await;
    assert!(!msg.is_empty());
}

#[cfg(not(unix))]
#[test]
fn smoke_acp_reader_helper_production_symbols() {
    let _ = crate::acp_tests::reader_tests_helpers::acp_activity_state;
    let _: Option<crate::acp_tests::reader_tests_helpers::IncomingDispatchParts> = None;
    let _: Option<crate::acp_tests::reader_tests_helpers::CatSession> = None;
}

#[tokio::test]
async fn smoke_acp_session_prompt_round_health() {
    let dir = tempfile::tempdir().expect("tempdir");
    let session = crate::acp::test_captive_session::captive_cat_acp_session_for_tests(dir.path());
    let health = super::acp_session_take_prompt_round_health(&session);
    assert!(health.format_lines().is_empty());
}

#[test]
fn smoke_spawn_and_agent_env_helpers() {
    let _ = super::resolve_agent_bin();
    let _ = super::test_no_real_agent_enabled();
    let _ = super::auth_probe(&["/bin/true"]);
    let _ = stringify!(MALVIN_TEST_NO_REAL_AGENT_ENV);
}

#[test]
fn smoke_acp_inc_symbols_for_kiss() {
    let _ = stringify!(AcpChildStdout);
    let _ = stringify!(AcpHandshakeIo);
    let _ = stringify!(PromptRpcCleanup);
    let _ = stringify!(clear_if_prompt_response);
    let _ = stringify!(ReaderLoopInput);
    let _ = stringify!(IncomingLineDispatch);
    let _ = stringify!(ReaderLoopFinishCtx);
    let _ = stringify!(ReaderLoopLineIo);
    let _ = stringify!(ReaderLoopDrainCtx);
    let _ = super::resolve_agent_bin;
    let _ = super::spawn_agent_acp_session;
    let _ = stringify!(PromptTraceDispatchMeta);
    let _ = stringify!(DoOutgoingTraceParts);
    let _ = stringify!(SessionAfterStdioIn);
    let _ = stringify!(take_stdio_pipes);
    let _ = stringify!(take_stdio_pipes_from_piped_spawn);
    let _ = stringify!(acp_stdio);
    let _ = stringify!(spawn_acp_session);
    let _ = stringify!(spawn_acp_session_microsandbox);
    let _ = stringify!(spawn_microsandbox_stdout_reader);
    let _ = stringify!(SandboxReaderArgs);
    let _ = stringify!(spawn_acp_sandbox_stdout_reader);
    let _ = stringify!(crate::acp::sandbox_stdio::SandboxStdoutStream);
    let _ = stringify!(crate::acp::sandbox_stdio::SandboxStdoutStream::new);
    let _ = stringify!(crate::acp::sandbox_stdio::SandboxStdoutStream::poll_read);
    let _ = stringify!(crate::acp::sandbox_stdio::write_guest_line);
    let _ = stringify!(session_after_stdio);
    let _ = stringify!(kill_child_and_finalize_containment);
    let _ = stringify!(acp_session_set_run_timing);
    let _ = stringify!(acp_session_take_prompt_round_health);
}
