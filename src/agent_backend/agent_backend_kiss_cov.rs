//! External kiss witnesses for `agent_backend` modules.

#[test]
fn kiss_witness_backend_ops() {
    let _ = super::backend_ops::agent_backend_set_run_timing;
    let _ = super::backend_ops::agent_backend_attach_run_timing_for_session;
    let _ = super::backend_ops::agent_backend_ensure_run_timing_for_session;
    let _ = super::backend_ops::agent_backend_ensure_coder_session;
    let _ = super::backend_ops::agent_backend_timing;
}

#[test]
fn ensure_run_timing_for_session_installs_when_missing() {
    let mut backend = super::backend::AgentBackend::CursorSdk(
        crate::cursor_sdk::CursorSdkClient::new(
            "cursor:auto".into(),
            super::test_support::test_io(),
        ),
    );
    assert!(super::backend_ops::agent_backend_timing(&backend).is_none());
    let timing = super::backend_ops::agent_backend_ensure_run_timing_for_session(&mut backend);
    let again = super::backend_ops::agent_backend_ensure_run_timing_for_session(&mut backend);
    assert!(std::sync::Arc::ptr_eq(&timing, &again));
}
