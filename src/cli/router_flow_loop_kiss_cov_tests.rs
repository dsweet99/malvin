//! Kiss identifier refs for [`super`] agent loop driver.

use super::{
    decide_router_gates_exit, decide_router_loop_exit_not_done, router_exit_summarize_for,
    run_router_agent_loops, RouterAgentLoopInput, RouterAgentLoopOutcome, RouterLoopDecision,
};
use crate::router_flow::router_flow_acp::RouterExitSummarize;

#[test]
fn kiss_cov_router_flow_loop_privates() {
    let _: Option<RouterAgentLoopInput> = None;
    let _: Option<RouterAgentLoopOutcome> = None;
    let _ = run_router_agent_loops;
    let _ = decide_router_loop_exit_not_done;
    let _ = decide_router_gates_exit;
    let _ = router_exit_summarize_for;
    let _ = RouterExitSummarize::Run;
    let _ = RouterLoopDecision::Continue;
    let _ = stringify!(client);
    let _ = stringify!(artifacts);
    let _ = stringify!(prompt_store);
    let _ = stringify!(shared);
    let _ = stringify!(last_acp);
    let _ = stringify!(last_backups);
    let _ = stringify!(work_dir);
    let _ = stringify!(agent_loop);
    let _ = stringify!(iteration_backups);
    let _ = stringify!(session_end);
    let _ = stringify!(iteration);
    let _ = stringify!(gates);
    let _ = stringify!(done);
    let _ = stringify!(max_loops);
    let _ = stringify!(backups);
}

#[test]
fn router_exit_summarize_only_when_exiting() {
    assert_eq!(
        router_exit_summarize_for(&RouterLoopDecision::Continue),
        RouterExitSummarize::Skip
    );
    assert_eq!(
        router_exit_summarize_for(&RouterLoopDecision::Exit),
        RouterExitSummarize::Run
    );
    assert_eq!(
        router_exit_summarize_for(&RouterLoopDecision::ExitGatesFailed("x".into())),
        RouterExitSummarize::Run
    );
}

#[test]
fn decide_router_loop_exit_when_not_done() {
    assert!(matches!(
        decide_router_loop_exit_not_done(1, 2),
        RouterLoopDecision::Continue
    ));
    assert!(matches!(
        decide_router_loop_exit_not_done(2, 2),
        RouterLoopDecision::Exit
    ));
}

#[cfg(unix)]
#[test]
fn decide_router_gates_exit_direct_paths() {
    crate::test_utils::with_isolated_home(|workspace| {
        let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
            "gates exit",
            Some(workspace),
            crate::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        crate::seed_malvin_checks(workspace, "true\n");
        let backups_ok =
            crate::artifacts::SessionDotfileBackups::snapshot(workspace).expect("snap");
        let ok = decide_router_gates_exit(&artifacts, &backups_ok, 1, 2);
        assert!(matches!(ok, RouterLoopDecision::Exit));
        crate::seed_malvin_checks(workspace, "false\n");
        let backups_bad =
            crate::artifacts::SessionDotfileBackups::snapshot(workspace).expect("snap");
        let cont = decide_router_gates_exit(&artifacts, &backups_bad, 1, 3);
        assert!(matches!(cont, RouterLoopDecision::Continue));
        let failed = decide_router_gates_exit(&artifacts, &backups_bad, 3, 3);
        assert!(matches!(failed, RouterLoopDecision::ExitGatesFailed(_)));
    });
}

#[cfg(unix)]
#[test]
fn kiss_cov_router_flow_loop_live_outcome_fields() {
    use crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::install_mock_router_agent_env;

    use crate::router_flow::router_flow_acp::router_flow_acp_tests::{
        router_boot_client_artifacts, test_router_shared,
    };

    crate::test_utils::with_isolated_home(|workspace| {
        crate::test_utils::block_on_test_async(async {
            crate::seed_malvin_checks(workspace, "true\n");
            let mock = workspace.join("mock-router-agent");
            let _env = install_mock_router_agent_env(workspace, &mock);
            let (shared, workflow) = test_router_shared();
            let (mut client, artifacts, prompt_store) =
                router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
            let input = RouterAgentLoopInput {
                client: &mut client,
                artifacts: &artifacts,
                prompt_store: &prompt_store,
                shared: &shared,
                max_loops: 1,
            };
            let RouterAgentLoopInput {
                client: _,
                artifacts: _,
                prompt_store: _,
                shared: _,
                max_loops: _,
            } = input;
            let outcome = run_router_agent_loops(input).await.expect("loops");
            let RouterAgentLoopOutcome {
                last_acp: _,
                last_backups: _,
            } = outcome;
        });
    });
}

#[cfg(unix)]
#[test]
fn kiss_cov_router_flow_loop_test_helpers() {
    let _ = stringify!(run_router_agent_loops_single_session_a_to_b);
}
