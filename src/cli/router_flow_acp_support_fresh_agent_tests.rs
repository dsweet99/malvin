use super::{router_iteration_log_path, run_router_turns, RouterAcpIterationInput};
use crate::cli::{RouterOpts, SharedOpts};
use malvin::backends::agent_backend::SdkClient;

#[test]
fn next_router_a_creates_a_new_agent_instead_of_resuming() {
    let boots = run_two_router_a_turns();
    assert_eq!(boots, (2, 0), "second router_a must create, not resume");
}

#[test]
fn router_summarize_stays_on_the_open_agent() {
    let boots = run_router_a_then_summarize();
    assert_eq!(boots, (1, 0), "summarize must not start another agent");
}

fn boot_counts(dir: &std::path::Path) -> (usize, usize) {
    let boots = std::fs::read_to_string(dir.join("boots")).unwrap_or_default();
    let creates = boots.lines().filter(|line| *line == "create").count();
    let resumes = boots.lines().filter(|line| *line == "resume").count();
    (creates, resumes)
}

fn run_two_router_a_turns() -> (usize, usize) {
    with_mock_router(|workspace, once_dir| {
        malvin::test_support::test_utils::block_on_test_async(async {
            let mut client = open_client(workspace);
            let _ = run_turns(&mut client, workspace, 1).await;
            let second = run_turns(&mut client, workspace, 2).await;
            assert!(
                second.contains("BEGIN MALVIN HEADER"),
                "second router_a must include header.md: {second}"
            );
            let _ = client.end_coder_session().await;
            boot_counts(once_dir)
        })
    })
}

fn run_router_a_then_summarize() -> (usize, usize) {
    with_mock_router(|workspace, once_dir| {
        malvin::test_support::test_utils::block_on_test_async(async {
            let mut client = open_client(workspace);
            let _ = run_turns(&mut client, workspace, 1).await;
            summarize(&mut client, workspace).await;
            boot_counts(once_dir)
        })
    })
}

fn with_mock_router(
    body: impl FnOnce(&std::path::Path, &std::path::Path) -> (usize, usize),
) -> (usize, usize) {
    malvin::backends::cursor_sdk::node_resolve::resolve_node_bin().expect("node");
    let mut counts = (0, 0);
    malvin::test_support::test_utils::with_isolated_home(|workspace| {
        counts = drive_mock_router(workspace, body);
    });
    counts
}

fn drive_mock_router(
    workspace: &std::path::Path,
    body: impl FnOnce(&std::path::Path, &std::path::Path) -> (usize, usize),
) -> (usize, usize) {
    let once_dir = workspace.join("once");
    std::fs::create_dir_all(&once_dir).expect("once dir");
    let _env = hold_mock_env(&once_dir);
    malvin::test_support::test_utils::enable_test_fast_teardown();
    body(workspace, &once_dir)
}

fn hold_mock_env(once_dir: &std::path::Path) -> malvin::test_support::test_utils::SavedEnvVars {
    let saved = malvin::test_support::test_utils::SavedEnvVars::capture(&[
        "MALVIN_CURSOR_SDK_BRIDGE",
        "CURSOR_API_KEY",
        "MALVIN_TEST_NO_REAL_AGENT",
        "MOCK_BRIDGE_DONE_WHEN_PROMPT_HAS",
        "MOCK_BRIDGE_ONCE_DIR",
    ]);
    let bridge = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/backends/cursor_sdk/mock_bridge.js");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_CURSOR_SDK_BRIDGE", bridge);
        std::env::set_var("CURSOR_API_KEY", "test-key");
        std::env::set_var("MALVIN_TEST_NO_REAL_AGENT", "1");
        std::env::set_var(
            "MOCK_BRIDGE_DONE_WHEN_PROMPT_HAS",
            "Find unsatisfied requirements",
        );
        std::env::set_var("MOCK_BRIDGE_ONCE_DIR", once_dir);
    }
    saved
}

fn open_client(workspace: &std::path::Path) -> SdkClient {
    let shared = SharedOpts::test_defaults();
    let io = malvin::agent_process::AgentIoOptions {
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: true,
    };
    let mut client = SdkClient::new(shared.model, io);
    client.prompts_log_run_dir = Some(workspace.to_path_buf());
    let _ = client.attach_run_timing_for_session();
    client
}

async fn run_turns(
    client: &mut SdkClient,
    workspace: &std::path::Path,
    agent_loop: usize,
) -> String {
    let log_path = finish_turn(client, workspace, agent_loop, false).await;
    std::fs::read_to_string(&log_path).unwrap_or_default()
}

async fn summarize(client: &mut SdkClient, workspace: &std::path::Path) {
    let _ = finish_turn(client, workspace, 1, true).await;
}

async fn finish_turn(
    client: &mut SdkClient,
    workspace: &std::path::Path,
    agent_loop: usize,
    summarize_now: bool,
) -> std::path::PathBuf {
    let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
        "fresh agent per router_a",
        Some(workspace),
        malvin::workspace::run_id::RunDirOptions::default(),
    )
    .expect("artifacts");
    let prompt_store = crate::cli::router_flow::prepare_router_prompt_store().expect("prompts");
    let shared = SharedOpts::test_defaults();
    let router = RouterOpts::test_defaults();
    let log_path = router_iteration_log_path(&artifacts, agent_loop);
    let mut input = RouterAcpIterationInput {
        client,
        artifacts: &artifacts,
        prompt_store: &prompt_store,
        shared: &shared,
        router: &router,
        agent_loop,
        session_end: if summarize_now {
            malvin::run_timing::acp_post_run::RunTimingSessionEnd::Finalize
        } else {
            malvin::run_timing::acp_post_run::RunTimingSessionEnd::AccumulateRun
        },
        max_hypotheses: 1,
    };
    if summarize_now {
        let timing = input.client.attach_run_timing_for_session();
        super::super::finalize_router_acp_iteration(
            &mut input,
            timing,
            super::RouterExitSummarize::Run,
        )
        .await
        .expect("summarize");
    } else {
        run_router_turns(&mut input, &log_path)
            .await
            .expect("router turns");
    }
    log_path
}
