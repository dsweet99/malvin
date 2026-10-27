use super::super::{RouterAgentLoopInput, run_router_models};
use super::{new_router_client, prepare_router_prompt_store, with_bridge};
use crate::cli::{RouterOpts, SharedOpts};
use malvin::test_support::test_utils::block_on_test_async;

fn check_terminal_case(models: &str, expected_attempts: &str, expect_success: bool) {
    with_bridge(|work| {
        let mut shared = SharedOpts::test_defaults();
        shared.model =
            malvin::config::malvin_config_file::parse_model_list_cli_arg(models).unwrap();
        let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
            "terminal fallback QA",
            Some(work),
            malvin::workspace::run_id::RunDirOptions::default(),
        )
        .unwrap();
        let store = prepare_router_prompt_store().unwrap();
        let mut client = new_router_client(&shared);
        let router = RouterOpts::test_defaults();
        let result = block_on_test_async(run_router_models(RouterAgentLoopInput {
            client: &mut client,
            artifacts: &artifacts,
            prompt_store: &store,
            shared: &shared,
            router: &router,
            max_loops: 1,
        }));
        let attempts = std::fs::read_to_string(work.join("attempts")).unwrap();
        assert_eq!(attempts, expected_attempts);
        assert_eq!(result.is_ok(), expect_success, "{result:?}");
        if let Err(error) = &result {
            assert!(error.contains("Insufficient funds"), "{error}");
        }
        let timing = std::fs::read_to_string(artifacts.run_dir.join("run_timing.json"));
        assert!(
            !client.has_open_coder_session(),
            "Router must close its session"
        );
        let report: serde_json::Value = serde_json::from_str(
            &timing.expect("billable completed turns must be persisted even on failure"),
        )
        .unwrap();
        let expected = if expect_success { 30 } else { 20 };
        assert_eq!(report["tokens"]["tokens_out"], expected / 10);
        assert_eq!(report["tokens"]["usage_tx_count"], expected / 10);
        assert_eq!(
            report["tokens"]["tokens_in"], expected,
            "Final usage must include every completed billable turn, including the final failed model's turns"
        );
    });
}

#[test]
fn terminal_summary_failure_after_fallback_preserves_usage() {
    check_terminal_case(
        "cursor:poor,cursor:summary-poor",
        "poor\nsummary-poor\n",
        false,
    );
}

#[test]
fn terminal_summary_failure_without_fallback_preserves_usage() {
    check_terminal_case("cursor:summary-poor", "summary-poor\n", false);
}

#[test]
fn successful_backup_control_persists_usage() {
    check_terminal_case("cursor:poor,cursor:good", "poor\ngood\n", true);
}

#[path = "router_flow_fallback_qa_tests.rs"]
mod qa_tests;
