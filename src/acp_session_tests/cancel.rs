use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::acp_session_unit_tests::session_inner::dead_transport_session_inner;

fn busy_session_with_dead_transport() -> crate::acp::AcpSession {
    crate::acp::AcpSession(Arc::new(dead_transport_session_inner()))
}

#[tokio::test]
async fn acp_session_cancel_clears_busy_state_after_rpc_error() {
    let session = busy_session_with_dead_transport();
    let err = session
        .cancel()
        .await
        .expect_err("cancel should fail on dead transport");
    assert!(err.contains("session is dead"), "{err}");
    assert!(!session.is_busy());
    assert_eq!(session.0.prompt_rpc_id.load(Ordering::SeqCst), 0);
    assert!(session.0.trace_writer.lock().await.is_none());
}
