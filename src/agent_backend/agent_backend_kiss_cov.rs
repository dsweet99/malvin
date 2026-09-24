use super::test_support::test_io;
use crate::model_id::ModelBackend;
use crate::model_id::parse_model_id;

#[test]
fn agent_backend_tracks_consecutive_errors_via_ops() {
    let model = parse_model_id("cursor:auto").expect("model");
    let mut backend = crate::agent_backend::new_cursor(model, test_io());
    assert!(!backend.backend_error_tracker().should_stop_and_exit());
    assert!(!backend.record_backend_error("test_err"));
    assert!(!backend.record_backend_error("test_err"));
    assert!(!backend.backend_error_tracker().should_stop_and_exit());
    assert!(backend.record_backend_error("test_err"));
    assert!(backend.backend_error_tracker().should_stop_and_exit());

    backend.record_backend_success();
    assert!(!backend.backend_error_tracker().should_stop_and_exit());
    assert_eq!(backend.backend_error_tracker().consecutive_count(), 0);
}

#[test]
fn ensure_run_timing_for_session_installs_when_missing() {
    let mut backend = crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", test_io(), 1);
    assert!(backend.timing.is_none());
    let timing = crate::agent_backend::ensure_run_timing_for_session(&mut backend);
    let again = crate::agent_backend::ensure_run_timing_for_session(&mut backend);
    assert!(std::sync::Arc::ptr_eq(&timing, &again));
}

#[test]
fn kiss_witness_unified_sdk_client_and_backend() {
    let model = parse_model_id("cursor:auto").expect("model");
    let cursor = crate::agent_backend::new_cursor(model, test_io());
    assert_eq!(cursor.model.canonical(), "cursor:auto");
    assert!(matches!(cursor.model.backend, ModelBackend::Cursor));

    let pi_model = parse_model_id("rpi:openai/gpt-4o").expect("pi");
    let pi = crate::agent_backend::new_pi(pi_model, test_io());
    assert!(matches!(pi.model.backend, ModelBackend::Pi));

    let codex_model = parse_model_id("codex:gpt-5.6").expect("codex");
    let codex = crate::agent_backend::sdk_client::new_codex(codex_model, test_io());
    assert!(matches!(codex.model.backend, ModelBackend::Codex));

    let mut backend = cursor;
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
    let _ = stringify!(Idle);
    let _ = stringify!(ActiveCoderSession);
    let _ = stringify!(active_coder_session);
    assert!(backend.coder.is_idle());
    assert!(backend.active_coder_session().is_err());
    let _ = stringify!(cursor_resume_id);
    let _ = stringify!(spawn_with_retries);
    let _ = stringify!(record_spawn_success);
    let _ = stringify!(spawn_service_wire);
    let _ = stringify!(CoderSessionEnsure);
    let _ = stringify!(start_coder_session);
    let _ = stringify!(bind_session_header);
    let _ = stringify!(SessionHeaderLifecycle);
    let _ = stringify!(deliver_session_header_if_needed);
    let _ = stringify!(emit_agent_started_log);
    let _ = stringify!(force_fresh_agent_for_retry);
    let _ = stringify!(DRAIN_IDLE_PREFIX_BRIDGE);
    let _ = stringify!(DRAIN_IDLE_PREFIX_NPM_PI);
    let _ = stringify!(DRAIN_IDLE_PREFIX_PI);
    let _ = stringify!(DRAIN_IDLE_PREFIX_CODEX);
    let _ = stringify!(prompts_log_run_dir);
    let _ = stringify!(BackendErrorTracker);
    let _ = stringify!(MAX_CONSECUTIVE_SAME_BACKEND_ERRORS);
    let _ = stringify!(format_backend_consecutive_error_message);
    let _ = stringify!(ensure_run_timing_for_session);
    let _ = stringify!(set_implement_display_name);
}
