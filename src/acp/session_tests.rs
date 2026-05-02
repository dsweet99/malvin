#[test]
fn kiss_stringify_acp_session_units() {
    let _ = stringify!(crate::acp::session::prompt_stdout_replacement);
    let _ = stringify!(crate::acp::session::rpc_session_prompt_text);
    let _ = stringify!(crate::acp::session::do_split_trace_preamble);
}

#[tokio::test]
#[allow(unsafe_code)]
#[allow(clippy::await_holding_lock)]
async fn acp_session_cancel_clears_busy_state_after_rpc_error() {
    use std::{sync::Arc, time::Duration};
    use std::sync::atomic::Ordering;
    use std::process::Stdio;

    let mut child = tokio::process::Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn cat");
    let stdin = child.stdin.take().expect("stdin");

    let session = crate::acp::AcpSession(Arc::new(crate::acp::session_types::AcpSessionInner {
        child: tokio::sync::Mutex::new(child),
        stdin: Arc::new(tokio::sync::Mutex::new(stdin)),
        pending: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::default())),
        acp_activity_seq: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        acp_activity_notify: Arc::new(tokio::sync::Notify::new()),
        next_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        session_id: "session-id".to_string(),
        reader_dead: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        rpc_timeout: Duration::from_millis(100),
        busy: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        trace_writer: Arc::new(tokio::sync::Mutex::new(None)),
        prompt_rpc_id: Arc::new(std::sync::atomic::AtomicU64::new(123)),
        prompt_singleflight: Arc::new(tokio::sync::Mutex::new(())),
        acp_verbose: false,
        ui_idle_notify: None,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
    }));

    let err = session.cancel().await.expect_err("cancel should fail on dead transport");
    assert!(err.contains("session is dead"), "{err}");
    assert!(!session.is_busy());
    assert_eq!(session.0.prompt_rpc_id.load(Ordering::SeqCst), 0);
    assert!(session.0.trace_writer.lock().await.is_none());
}
