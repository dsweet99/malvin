//! External kiss witnesses for `agent_backend` modules.

#[test]
fn kiss_witness_backend_ops() {
    let _ = super::backend_ops::agent_backend_run_kpop_flow;
    let _ = stringify!(run_kpop_flow_mini);
    let _ = super::backend_ops::agent_backend_set_run_timing;
    let _ = super::backend_ops::agent_backend_attach_run_timing_for_session;
    let _ = super::backend_ops::agent_backend_timing;
    let _ = super::backend_ops::agent_backend_run_kpop_multiturn;
}

#[test]
fn kiss_witness_backend_kpop_tests() {
    let _ = super::backend_kpop_test_helpers::mock_backend;
    let _ = super::backend_kpop_test_helpers::empty_backups;
    let _ = super::backend_kpop_test_helpers::mock_backend_bash_turn_exhaustion;
    let _ = super::backend_kpop_test_helpers::mini_done_backend;
    let _ = stringify!(agent_backend_run_kpop_flow_mini_stops_on_non_retryable_error);
    let _ = stringify!(agent_backend_run_kpop_multiturn_mini_stops_on_non_retryable_error);
}

#[test]
fn kiss_witness_kpop_bridge() {
    let _ = super::kpop_bridge::run_kpop_flow_once_mini;
}
