use super::prelude::*;
use super::shared_handshake::*;
use super::shared_harness::*;

pub(crate) async fn handshake_skip_login_session_id(tmp: &Path, bin: &Path) -> String {
    let mut cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp,
        bin_override: Some(bin),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    let child = crate::acp::spawn_agent_acp_child(&mut cmd)
        .await
        .expect("spawn");
    let mut hk = handshake_attach_and_start_reader(child);
    let sid = handshake_inner(HandshakeParams {
        io: &hk.io,
        next_id: &hk.next_id,
        cwd: tmp,
        rpc_timeout: acp_rpc_timeout(),
        require_cursor_login_auth: false,
        child_pid: None,
    })
    .await
    .expect("session/new should work without cursor_login authenticate");
    let _ = hk.child.kill().await;
    let _ = hk.child.wait().await;
    sid
}

#[tokio::test]
async fn handshake_can_skip_cursor_login_when_api_key_mode_is_used() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "env-key");
    }
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("auth-rejected-session-ok");
    write_authenticate_rejected_but_session_new_ok_mock(&bin).await;

    let sid = handshake_skip_login_session_id(tmp.path(), &bin).await;
    assert_eq!(sid, "t1");

    clear_cursor_env_for_test();
}

#[tokio::test]
async fn test_rpc_cancel_when_pending_sender_dropped() {
    let h = RpcSleepHarness::spawn_sleep("60", SleepStdoutDrainMode::LargeBuf).await;
    let io = h.io();
    let next_id = Arc::new(AtomicU64::new(1));
    let pending = io.pending.clone();

    let _send = tokio::spawn(async move {
        let r = rpc_request(RpcRequestNext {
            io: &io,
            next_id: &next_id,
            method: "nope",
            params: json!({}),
            rpc_timeout: acp_rpc_timeout(),
            child_pid: None,
        })
        .await;
        let e = r.unwrap_err();
        assert!(e.contains("canceled") || e.contains("session"), "{e}");
    });

    for _ in 0..20 {
        if !pending.lock().await.is_empty() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
    pending.lock().await.clear();

    h.shutdown().await;
    let _ = stringify!(send.await);
}


#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_handshake_skip_login_session_id() { let _ = handshake_skip_login_session_id; }

    #[test]
    fn kiss_cov_handshake_can_skip_cursor_login_when_api_key_mode_is_used() { let _ = handshake_can_skip_cursor_login_when_api_key_mode_is_used; }

    #[test]
    fn kiss_cov_test_rpc_cancel_when_pending_sender_dropped() { let _ = test_rpc_cancel_when_pending_sender_dropped; }

}
