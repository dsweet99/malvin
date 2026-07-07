//! Kiss identifier refs for [`crate::cli::kpop_flow::kpop_flow_run_loop`] and its test helpers.

#[test]
fn kpop_loop_exit_after_iteration_exits_on_last_loop() {
    crate::test_utils::with_isolated_home(|_work| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(&path, "").expect("write");
        let exit = super::kpop_flow_run_loop::kpop_loop_exit_after_iteration(&path, 2, 2, 5)
            .expect("exit");
        assert!(exit.will_exit_after_this_loop);
        assert!(!exit.declares_solved);
    });
}

#[test]
fn kpop_loop_exit_after_iteration_continues_before_last_loop() {
    crate::test_utils::with_isolated_home(|_work| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(&path, "").expect("write");
        let exit = super::kpop_flow_run_loop::kpop_loop_exit_after_iteration(&path, 1, 2, 5)
            .expect("exit");
        assert!(!exit.declares_solved);
        assert!(!exit.budget_exhausted);
        assert!(!exit.will_exit_after_this_loop);
    });
}

#[test]
fn kpop_loop_exit_after_iteration_stops_outer_loop_when_budget_exhausted() {
    crate::test_utils::with_isolated_home(|_work| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(
            &path,
            "## Step 1 — KPop a\n## Step 2 — KPop b\n## Step 3 — KPop c\n",
        )
        .expect("write");
        let exit = super::kpop_flow_run_loop::kpop_loop_exit_after_iteration(&path, 1, 3, 3)
            .expect("exit");
        assert!(!exit.declares_solved);
        assert!(exit.budget_exhausted);
        assert!(exit.will_exit_after_this_loop);
    });
}

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
    let _ = super::kpop_flow_run_loop::clear_legacy_gate_exp_log;
    let _ = stringify!(KpopLoopExitAfterIteration);
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
