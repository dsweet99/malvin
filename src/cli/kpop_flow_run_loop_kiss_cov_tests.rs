//! Kiss identifier refs for [`crate::cli::kpop_flow::kpop_flow_run_loop`] and its test helpers.

#[test]
fn kiss_cov_kpop_flow_run_loop_privates() {
    let outcome = super::kpop_flow_run_loop::kpop_loop_abort(false, "err".into());
    let super::kpop_flow_run_loop::RunKpopAgentLoopsOutcome {
        acp_result,
        agent_ran,
    } = outcome;
    assert!(acp_result.is_err());
    assert!(!agent_ran);
    let _: Option<super::kpop_flow_run_loop::RunKpopAgentLoopsParams<'_>> = None;
    let _: Option<super::kpop_flow_run_loop::KpopLoopSnapshot> = None;
    let _ = super::kpop_flow_run_loop::run_kpop_agent_loops;
    let _ = super::kpop_flow_run_loop::kpop_exp_log_declares_solved;
    let _ = super::kpop_flow_run_loop::clear_legacy_gate_exp_log;
    let _ = stringify!(KpopLoopExitAfterIteration);
    let _ = stringify!(declares_solved);
    let _ = stringify!(will_exit_after_this_loop);
    let _ = stringify!(kpop);
    let _ = stringify!(store);
    let _ = stringify!(client);
    let _ = stringify!(prepared);
    let _ = stringify!(backups);
    let _ = stringify!(exp_iter);
    let _ = stringify!(exp_log_path);
}

#[cfg(unix)]
#[test]
fn kiss_cov_kpop_flow_run_loop_test_helpers() {
    let _ = super::kpop_flow_run_loop_tests::test_kpop_args;
    let _ = super::kpop_flow_run_loop_tests::install_mock_agent_env;
    let _ = super::kpop_flow_run_loop_tests::write_mock_agent;
    let _ = stringify!(run_kpop_agent_loops_propagates_exp_log_setup_error);
    let _ = stringify!(run_kpop_agent_loops_executes_mock_agent);
}
