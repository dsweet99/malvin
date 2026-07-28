//! Tests for [`super`] ACP iteration helpers.

use super::{run_router_acp_open_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome};
use crate::agent_backend::build_agent_backend;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::prompts::PromptStore;
use crate::router_flow::router_flow_prompt::{build_router_coder_run, prepare_router_prompt_store};
use crate::run_timing::acp_post_run::RunTimingSessionEnd;

pub(crate) fn test_router_shared() -> (SharedOpts, WorkflowCliOptions) {
    let shared = SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: true,
        gates: false,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        no_download: false,
        git: false,
    };
    let workflow = WorkflowCliOptions { force: false };
    (shared, workflow)
}

pub(crate) fn router_boot_client_artifacts(
    workspace: &std::path::Path,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<
    (
        crate::agent_backend::AgentBackend,
        crate::artifacts::RunArtifacts,
        crate::router_flow::router_flow_prompt::RouterCoderRun,
        PromptStore,
    ),
    String,
> {
    let client = build_agent_backend(
        shared,
        workflow,
        shared.acp_stdout_markdown_enabled(),
        "router",
    )?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
        "investigate task",
        Some(workspace),
        crate::run_id::RunDirOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    let prompt_store = prepare_router_prompt_store()?;
    let coder = build_router_coder_run(
        &artifacts,
        "investigate task",
        crate::workflow_context::PromptModelOpts::new(crate::config::DEFAULT_CLI_MODEL, false),
    )?;
    Ok((client, artifacts, coder, prompt_store))
}

#[test]
fn kiss_cov_router_acp_iteration_input_type_name() {
    let _ = std::any::type_name::<RouterAcpIterationInput<'_>>();
    let _ = std::any::type_name::<RouterAcpIterationOutcome>();
    let _ = RunTimingSessionEnd::Finalize;
}

#[cfg(unix)]
mod unix_cov {
    use super::super::router_flow_acp_mock_tests::{
        install_mock_router_agent_env, install_mock_router_agent_env_with_script,
        write_mock_router_agent_missing_requirements, write_mock_router_agent_session_fail,
    };
    use super::{
        router_boot_client_artifacts, run_router_acp_open_iteration, test_router_shared,
        RouterAcpIterationInput, RouterAcpIterationOutcome, RunTimingSessionEnd,
    };
    use crate::router_flow::router_flow_loop::{run_router_agent_loops, RouterAgentLoopInput};

    #[test]
    fn run_router_acp_iteration_executes_mock_agent_full_sequence() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent");
                let _env = install_mock_router_agent_env(workspace, &mock);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let outcome = run_router_agent_loops(RouterAgentLoopInput {
                    client: &mut client,
                    artifacts: &artifacts,
                    coder: &coder,
                    prompt_store: &prompt_store,
                    shared: &shared,
                    max_loops: 1,
                })
                .await
                .expect("loops");
                outcome.last_acp.expect("acp");
                assert!(artifacts.log_path("router_1").is_file());
                let log_text =
                    std::fs::read_to_string(artifacts.log_path("router_1")).expect("read router log");
                assert!(
                    log_text.contains("router_requirements phase")
                        && log_text.contains("router_kpop phase")
                        && log_text.contains("router_work done")
                        && log_text.contains("router_summarize done"),
                    "router_1.log must retain requirements → kpop → work → summarize; got: {log_text}"
                );
                assert!(crate::artifacts::review_requirements_json(&artifacts).is_file());
                let counts_path = workspace.join(
                    super::super::router_flow_acp_mock_tests::ROUTER_MOCK_SESSION_COUNTS_FILE,
                );
                let counts_raw =
                    std::fs::read_to_string(&counts_path).expect("read mock session counts");
                let counts: serde_json::Value =
                    serde_json::from_str(&counts_raw).expect("parse mock session counts");
                assert_eq!(
                    counts.get("begins").and_then(serde_json::Value::as_u64),
                    Some(1),
                    "exactly one session/new (begin) around the full sequence: {counts_raw}"
                );
                assert_eq!(
                    counts.get("prompts").and_then(serde_json::Value::as_u64),
                    Some(4),
                    "requirements + KPop + work + summarize on one begin: {counts_raw}"
                );
                assert!(
                    workspace.join(".malvin_router_mock_saw_summarize").is_file(),
                    "mock must observe router_summarize.md body on the open session"
                );
            });
        });
    }

    #[test]
    fn run_router_acp_iteration_aborts_when_requirements_json_missing() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent-bad");
                write_mock_router_agent_missing_requirements(&mock);
                let _env = install_mock_router_agent_env_with_script(workspace, &mock);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAcpIterationOutcome { acp_result, .. } =
                    run_router_acp_open_iteration(RouterAcpIterationInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        coder: &coder,
                        prompt_store: &prompt_store,
                        shared: &shared,
                        agent_loop: 1,
                        session_end: RunTimingSessionEnd::Finalize,
                    })
                    .await;
                assert!(acp_result.is_err());
            });
        });
    }

    #[test]
    fn run_router_acp_iteration_propagates_begin_session_failure() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent-fail");
                write_mock_router_agent_session_fail(&mock);
                let _env = install_mock_router_agent_env_with_script(workspace, &mock);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAcpIterationOutcome { acp_result, .. } =
                    run_router_acp_open_iteration(RouterAcpIterationInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        coder: &coder,
                        prompt_store: &prompt_store,
                        shared: &shared,
                        agent_loop: 1,
                        session_end: RunTimingSessionEnd::Finalize,
                    })
                    .await;
                assert!(acp_result.is_err());
            });
        });
    }
}

#[cfg(unix)]
#[test]
fn kiss_cov_unix_cov_test_names() {
    let _ = stringify!(run_router_acp_iteration_executes_mock_agent_full_sequence);
    let _ = stringify!(run_router_acp_iteration_aborts_when_requirements_json_missing);
    let _ = stringify!(run_router_acp_iteration_propagates_begin_session_failure);
}
