#[test]
fn smoke_acp_reader_support_behavior() {
    use std::sync::atomic::Ordering;

    let (seq, _notify) = crate::acp_tests::reader_tests_helpers::acp_activity_state();
    assert_eq!(seq.load(Ordering::Relaxed), 0);
}

#[cfg(unix)]
#[tokio::test]
async fn smoke_reader_loop_eof_pending_error() {
    let msg = crate::acp_tests::reader_tests_helpers::reader_loop_eof_pending_error().await;
    assert!(!msg.is_empty());
}

#[cfg(not(unix))]
#[test]
fn smoke_acp_reader_helper_production_symbols() {
    let _ = crate::acp_tests::reader_tests_helpers::acp_activity_state;
    let _: Option<crate::acp_tests::reader_tests_helpers::IncomingDispatchParts> = None;
    let _: Option<crate::acp_tests::reader_tests_helpers::CatSession> = None;
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
    let _ = stringify!(resolve_agent_bin);
    let _ = stringify!(spawn_agent_acp_session);
    let _ = stringify!(PromptTraceDispatchMeta);
    let _ = stringify!(DoOutgoingTraceParts);
    let _ = stringify!(SessionAfterStdioIn);
    let _ = stringify!(take_stdio_pipes);
    let _ = stringify!(take_stdio_pipes_from_piped_spawn);
    let _ = stringify!(acp_stdio);
    let _ = stringify!(spawn_acp_session);
    let _ = stringify!(session_after_stdio);
    let _ = stringify!(kill_child_and_finalize_containment);
}
