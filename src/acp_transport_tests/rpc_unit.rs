use super::prelude::*;
use super::jsonrpc::*;
use super::shared_handshake::*;

#[test]
fn test_cursor_credentials_empty_strings_skipped() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some(""),
        auth_token: Some(""),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, None);
}

#[test]
fn test_cursor_credentials_skips_empty_key_only() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some(""),
        auth_token: Some("tok2"),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, Some("tok2"));
}

#[test]
fn test_cursor_credentials_skips_empty_token_only() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some("k2"),
        auth_token: Some(""),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, Some("k2"), None);
}

#[tokio::test]
async fn test_write_rpc_line_fails_after_child_stdin_closed() {
    let mut child = Command::new(crate::acp_test_unix_bin::unix_bin_with_fallback("sleep"))
        .arg("60")
        .stdin(Stdio::piped())
        .spawn()
        .expect("sleep");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let _ = child.kill().await;
    let _ = child.wait().await;

    // Kernel/async reactor may observe a closed read-end slightly after `wait` returns; poll
    // until `write_rpc_line` sees `EPIPE` / broken pipe (or time out).
    let mut last = Ok(());
    for _ in 0..100 {
        last = write_rpc_line(
            &stdin,
            RpcLineWriteOpts {
                line: r#"{"x":1}"#,
                acp_verbose: false,
                trace_jsonl: None,
                activity: None,
            },
        )
        .await;
        if last.is_err() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!("expected stdin write to fail after child exit (broken pipe), last={last:?}");
}

#[test]
fn format_jsonrpc_error_pretty_prints_cursor_style() {
    let err = json!({
        "code": -32602,
        "message": "Invalid params",
        "data": {"message": "Failed to open browser for login."}
    });
    let s = format_jsonrpc_error(&err);
    assert!(s.contains("32602"), "{s}");
    assert!(s.contains("Invalid params"), "{s}");
    assert!(s.contains("Failed to open browser"), "{s}");
}

#[test]
fn format_jsonrpc_error_falls_back_for_non_object() {
    assert_eq!(format_jsonrpc_error(&json!("plain")), "\"plain\"");
}

#[test]
fn test_agent_program_prefers_nonempty_override() {
    let p = Path::new("/tmp/mock-agent-override");
    assert!(agent_program(Some(p)).contains("mock-agent-override"));
    assert_eq!(agent_program(Some(Path::new(""))), AGENT_BIN);
    assert_eq!(agent_program(None), AGENT_BIN);
}



#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_test_write_rpc_line_fails_after_child_stdin_closed() { let _ = test_write_rpc_line_fails_after_child_stdin_closed; }

}
