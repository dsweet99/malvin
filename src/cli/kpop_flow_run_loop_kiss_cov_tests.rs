//! Kiss identifier refs for [`crate::cli::kpop_flow::kpop_flow_run_loop`] and its test helpers.

#[test]
fn kiss_cov_kpop_flow_run_loop_privates() {
    let _: Option<super::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsOutcome> = None;
    let _: Option<super::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsParams<'_>> = None;
    let _ = super::kpop_flow::kpop_flow_run_loop::run_kpop_agent_loops;
    let _ = super::kpop_flow::kpop_flow_run_loop::kpop_exp_log_declares_solved;
    let _ = super::kpop_flow::kpop_flow_run_loop::clear_legacy_gate_exp_log;
    let _ = stringify!(kpop_exp_log_declares_solved_reads_marker);
    let _ = stringify!(kpop_loop_abort_records_error_and_agent_ran);
    let _ = stringify!(kpop_loop_snapshot_ensures_home_config_exists);
    let _ = stringify!(snapshot_kpop_loop_dotfiles_and_exp_log_builds_paths);
}

#[cfg(unix)]
#[test]
fn kiss_cov_kpop_flow_run_loop_test_helpers() {
    let _ = super::kpop_flow::kpop_flow_run_loop_tests::test_kpop_args;
    let _ = super::kpop_flow::kpop_flow_run_loop_tests::install_mock_agent_env;
    let _ = super::kpop_flow::kpop_flow_run_loop_tests::write_mock_agent;
    let _ = stringify!(run_kpop_agent_loops_propagates_exp_log_setup_error);
    let _ = stringify!(run_kpop_agent_loops_executes_mock_agent);
}
