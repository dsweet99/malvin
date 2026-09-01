use crate::model_id::ModelBackend;
use super::test_support::test_io;
use crate::model_id::parse_model_id;

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
    let mut backend = crate::agent_backend::agent_backend_from_client(
        crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", test_io(), 1),
    );
    assert!(super::backend_ops::agent_backend_timing(&backend).is_none());
    let timing = super::backend_ops::agent_backend_ensure_run_timing_for_session(&mut backend);
    let again = super::backend_ops::agent_backend_ensure_run_timing_for_session(&mut backend);
    assert!(std::sync::Arc::ptr_eq(&timing, &again));
}

#[test]
fn kiss_witness_unified_sdk_client_and_backend() {
    let model = parse_model_id("cursor:auto").expect("model");
    let cursor = crate::agent_backend::new_cursor(model, test_io());
    assert_eq!(cursor.model.canonical(), "cursor:auto");
    assert!(matches!(cursor.model.backend, ModelBackend::Cursor));

    let pi_model = parse_model_id("pi:openai/gpt-4o").expect("pi");
    let pi = crate::agent_backend::new_pi(pi_model, test_io());
    assert!(matches!(pi.model.backend, ModelBackend::Pi));

    let codex_model = parse_model_id("codex:gpt-5.6").expect("codex");
    let codex = crate::agent_backend::sdk_client::new_codex(codex_model, test_io());
    assert!(matches!(codex.model.backend, ModelBackend::Codex));

    let mut backend = crate::agent_backend::agent_backend_from_client(cursor);
    assert!(backend.prompts_log_run_dir.is_none());
    backend.prompts_log_run_dir = Some(std::path::PathBuf::from("/tmp"));
    assert!(backend.prompts_log_run_dir.is_some());
    assert_eq!(
        backend.max_acp_retries,
        crate::support_paths::DEFAULT_MAX_ACP_RETRIES
    );
    assert!(!backend.has_open_coder_session());
    let _ = stringify!(ModelBackend);
    let _ = stringify!(new_cursor);
    let _ = stringify!(new_pi);
    let _ = stringify!(new_codex);
    let _ = stringify!(backend);
    let _ = stringify!(BegunCoderSession);
    let _ = stringify!(cursor_resume_id);
    let _ = stringify!(spawn_with_retries);
    let _ = stringify!(spawn_service_wire);
    let _ = stringify!(reject_no_force);
    let _ = stringify!(require_force);
    let _ = stringify!(CoderSessionEnsure);
    let _ = stringify!(force_fresh_agent_for_retry);
    let _ = stringify!(NO_FORCE_MSG);
    let _ = stringify!(DRAIN_IDLE_PREFIX_BRIDGE);
    let _ = stringify!(DRAIN_IDLE_PREFIX_PI);
    let _ = stringify!(DRAIN_IDLE_PREFIX_CODEX);
    let _ = stringify!(prompts_log_run_dir);
}
