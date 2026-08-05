//! External kiss witnesses for `agent_backend` modules.

#[test]
fn kiss_witness_backend_ops() {
    let _ = super::backend_ops::agent_backend_set_run_timing;
    let _ = super::backend_ops::agent_backend_attach_run_timing_for_session;
    let _ = super::backend_ops::agent_backend_ensure_run_timing_for_session;
    let _ = super::backend_ops::agent_backend_timing;
}

#[test]
fn ensure_run_timing_for_session_installs_when_missing() {
    let mut backend = super::backend_kpop_test_helpers::mini_done_backend();
    assert!(super::backend_ops::agent_backend_timing(&backend).is_none());
    let timing = super::backend_ops::agent_backend_ensure_run_timing_for_session(&mut backend);
    let again = super::backend_ops::agent_backend_ensure_run_timing_for_session(&mut backend);
    assert!(std::sync::Arc::ptr_eq(&timing, &again));
}

#[test]
fn kiss_witness_backend_test_helpers() {
    let _ = super::backend_kpop_test_helpers::mock_backend;
    let _ = super::backend_kpop_test_helpers::empty_backups;
    let _ = super::backend_kpop_test_helpers::mock_backend_bash_turn_exhaustion;
    let _ = super::backend_kpop_test_helpers::mini_done_backend;
}

#[test]
fn kiss_witness_backend_contract_tests() {
    let _ = (
        super::backend_contract_tests::openrouter_transport_backend,
        super::backend_contract_tests::agent_backend_auth_shapes_mini_mock_http_and_acp,
        super::backend_contract_tests::agent_backend_mini_mock_lifecycle_and_prompt_without_begin,
        super::backend_contract_tests::agent_backend_log_dir_accessors_round_trip_both_variants,
        super::backend_contract_tests::kiss_cov_agent_backend_contract_symbols,
    );
}
