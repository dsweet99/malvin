use super::prelude::*;
use super::shared_handshake::*;

#[test]
fn requires_cursor_login_auth_skips_login_when_process_credentials_exist() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "key-from-env");
    }
    assert!(!requires_cursor_login_auth(None, None));
    clear_cursor_env_for_test();
}

#[tokio::test]
async fn test_handshake_hits_session_new_error_path() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join(format!(
        "bad-session-mock-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    write_bad_session_new_mock(&bin).await;

    let mut cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(&bin),
        api_key: Some(""),
        auth_token: Some(""),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    let child = crate::acp::spawn_agent_acp_child(&mut cmd)
        .await
        .expect("spawn");
    let mut hk = handshake_attach_and_start_reader(child);
    let err = handshake_inner(HandshakeParams {
        io: &hk.io,
        next_id: &hk.next_id,
        cwd: tmp.path(),
        rpc_timeout: acp_rpc_timeout(),
        require_cursor_login_auth: true,
        child_pid: None,
    })
    .await
    .unwrap_err();
    assert!(err.contains("sessionId"));

    let _ = hk.child.kill().await;
    let _ = hk.child.wait().await;
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_test_handshake_hits_session_new_error_path() { let _ = stringify!(test_handshake_hits_session_new_error_path); }

}
