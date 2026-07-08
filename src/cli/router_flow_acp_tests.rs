//! Tests for [`super`] ACP iteration helpers.

use super::{run_router_acp_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome};
use crate::router_flow::router_flow_prompt::{build_router_b_prompt_for_run, build_router_coder_run};
use crate::agent_backend::build_agent_backend;
use crate::cli::{SharedOpts, WorkflowCliOptions};

pub(crate) fn test_router_shared() -> (SharedOpts, WorkflowCliOptions) {
    let shared = SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: true,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini: false,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
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
        String,
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
    let coder = build_router_coder_run(&artifacts, "investigate task")?;
    let router_b_prompt = build_router_b_prompt_for_run(&artifacts)?;
    Ok((client, artifacts, coder, router_b_prompt))
}

#[test]
fn kiss_cov_router_acp_iteration_input_type_name() {
    let _ = std::any::type_name::<RouterAcpIterationInput<'_>>();
    let _ = std::any::type_name::<RouterAcpIterationOutcome>();
}

#[cfg(unix)]
mod unix_cov {
    use super::super::router_flow_acp_mock_tests::{
        install_mock_router_agent_env, install_mock_router_agent_env_with_script,
        write_mock_router_agent_session_fail,
    };
    use super::{
        router_boot_client_artifacts, run_router_acp_iteration, test_router_shared,
        RouterAcpIterationInput, RouterAcpIterationOutcome,
    };
    use crate::run_timing::acp_post_run::RunTimingSessionEnd;

    #[test]
    fn run_router_acp_iteration_executes_mock_agent_without_continue() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent");
                let _env = install_mock_router_agent_env(workspace, &mock, false);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, router_b_prompt) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAcpIterationOutcome {
                    acp_result,
                    wants_continue,
                } = run_router_acp_iteration(RouterAcpIterationInput {
                    client: &mut client,
                    artifacts: &artifacts,
                    coder: &coder,
                    router_b_prompt: &router_b_prompt,
                    session_end: RunTimingSessionEnd::Finalize,
                })
                .await;
                acp_result.expect("acp");
                assert!(!wants_continue);
            });
        });
    }

    #[test]
    fn run_router_acp_iteration_wants_continue_when_router_b_emits_marker() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent");
                let _env = install_mock_router_agent_env(workspace, &mock, true);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, router_b_prompt) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAcpIterationOutcome {
                    acp_result,
                    wants_continue,
                } = run_router_acp_iteration(RouterAcpIterationInput {
                    client: &mut client,
                    artifacts: &artifacts,
                    coder: &coder,
                    router_b_prompt: &router_b_prompt,
                    session_end: RunTimingSessionEnd::AccumulateRun,
                })
                .await;
                acp_result.expect("acp");
                assert!(wants_continue);
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
                let (mut client, artifacts, coder, router_b_prompt) =
                    router_boot_client_artifacts(workspace, &shared, workflow)
                        .expect("boot");
                let RouterAcpIterationOutcome {
                    acp_result,
                    wants_continue,
                } = run_router_acp_iteration(RouterAcpIterationInput {
                    client: &mut client,
                    artifacts: &artifacts,
                    coder: &coder,
                    router_b_prompt: &router_b_prompt,
                    session_end: RunTimingSessionEnd::Finalize,
                })
                .await;
                assert!(acp_result.is_err());
                assert!(!wants_continue);
            });
        });
    }
}

#[cfg(unix)]
#[test]
fn kiss_cov_unix_cov_test_names() {
    let _ = stringify!(run_router_acp_iteration_executes_mock_agent_without_continue);
    let _ = stringify!(run_router_acp_iteration_wants_continue_when_router_b_emits_marker);
    let _ = stringify!(run_router_acp_iteration_propagates_begin_session_failure);
}
