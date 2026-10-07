use super::{router_iteration_log_path, run_router_turns, RouterAcpIterationInput};
use crate::cli::{RouterOpts, SharedOpts};
use malvin::run_timing::acp_post_run::RunTimingSessionEnd;

const DONE_NEEDLE_ENV: &str = "MOCK_BRIDGE_DONE_WHEN_PROMPT_HAS";

struct MockRouterTurns {
    done: bool,
    router_log: String,
}

fn mock_bridge_js() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/backends/cursor_sdk/mock_bridge.js")
}

fn cache_node_bin_while_real_home_visible() {
    malvin::backends::cursor_sdk::node_resolve::resolve_node_bin().expect("node for mock bridge");
}

fn run_mock_router_turns(done_needle: &str) -> MockRouterTurns {
    cache_node_bin_while_real_home_visible();
    let mut outcome = None;
    malvin::test_support::test_utils::with_isolated_home(|workspace| {
        let _env = malvin::test_support::test_utils::SavedEnvVars::capture(&[
            "MALVIN_CURSOR_SDK_BRIDGE",
            "CURSOR_API_KEY",
            "MALVIN_TEST_NO_REAL_AGENT",
            DONE_NEEDLE_ENV,
        ]);
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("MALVIN_CURSOR_SDK_BRIDGE", mock_bridge_js());
            std::env::set_var("CURSOR_API_KEY", "test-key");
            std::env::set_var(DONE_NEEDLE_ENV, done_needle);
        }
        malvin::test_support::test_utils::enable_test_fast_teardown();
        outcome = Some(malvin::test_support::test_utils::block_on_test_async(
            drive_router_turns(workspace),
        ));
    });
    outcome.expect("router turns ran")
}

async fn drive_router_turns(workspace: &std::path::Path) -> MockRouterTurns {
    let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
        "done marker test",
        Some(workspace),
        malvin::workspace::run_id::RunDirOptions::default(),
    )
    .expect("artifacts");
    let prompt_store =
        crate::cli::router_flow::prepare_router_prompt_store().expect("prompt store");
    let shared = SharedOpts::test_defaults();
    let router = RouterOpts::test_defaults();
    let io = malvin::agent_process::AgentIoOptions {
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    };
    let mut client = malvin::backends::agent_backend::SdkClient::new(shared.model.clone(), io);
    let log_path = router_iteration_log_path(&artifacts, 1);
    let mut input = RouterAcpIterationInput {
        client: &mut client,
        artifacts: &artifacts,
        prompt_store: &prompt_store,
        shared: &shared,
        router: &router,
        agent_loop: 1,
        session_end: RunTimingSessionEnd::AccumulateRun,
        max_hypotheses: 1,
    };
    let done = run_router_turns(&mut input, &log_path)
        .await
        .expect("router turns")
        .done;
    let _ = client.end_coder_session().await;
    MockRouterTurns {
        done,
        router_log: std::fs::read_to_string(&log_path).unwrap_or_default(),
    }
}

#[test]
fn done_marker_in_router_a_reply_skips_router_b() {
    let turns = run_mock_router_turns("Find unsatisfied requirements");
    assert!(turns.done);
    assert!(
        !turns.router_log.contains("router_b"),
        "{}",
        turns.router_log
    );
}

#[test]
fn done_marker_in_router_b_reply_stops_loop() {
    let turns = run_mock_router_turns("KPop: Satisfy the requirements");
    assert!(
        turns.router_log.contains("router_b"),
        "{}",
        turns.router_log
    );
    assert!(turns.done);
}

#[test]
fn no_done_marker_continues_loop() {
    let turns = run_mock_router_turns("no prompt contains this needle");
    assert!(
        turns.router_log.contains("router_b"),
        "{}",
        turns.router_log
    );
    assert!(!turns.done);
}
