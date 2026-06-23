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
    let _ = stringify!(mock_backend);
    let _ = stringify!(empty_backups);
}

#[test]
fn kiss_witness_kpop_bridge() {
    let _ = super::kpop_bridge::run_kpop_flow_once_mini;
}
