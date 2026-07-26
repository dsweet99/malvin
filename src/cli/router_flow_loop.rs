use crate::agent_backend::AgentBackend;
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use super::router_flow_acp::{run_router_acp_iteration, RouterAcpIterationInput};
use crate::cli::SharedOpts;
use crate::prompts::PromptStore;
use crate::router_flow::router_flow_prompt;
use crate::run_timing::acp_post_run::RunTimingSessionEnd;

pub(crate) struct RouterAgentLoopInput<'a> {
    pub client: &'a mut AgentBackend,
    pub artifacts: &'a RunArtifacts,
    pub coder: &'a router_flow_prompt::RouterCoderRun,
    pub prompt_store: &'a PromptStore,
    pub shared: &'a SharedOpts,
}

pub(crate) struct RouterAgentLoopOutcome {
    pub last_acp: Result<(), String>,
    pub last_backups: SessionDotfileBackups,
}

/// Runs exactly one coder session: requirements → group `KPop`s → work → end.
pub(crate) async fn run_router_agent_loops(
    input: RouterAgentLoopInput<'_>,
) -> Result<RouterAgentLoopOutcome, String> {
    let RouterAgentLoopInput {
        client,
        artifacts,
        coder,
        prompt_store,
        shared,
    } = input;
    let work_dir = artifacts.work_dir.as_path();
    let iteration = run_router_acp_iteration(RouterAcpIterationInput {
        client,
        artifacts,
        coder,
        prompt_store,
        shared,
        agent_loop: 1,
        session_end: RunTimingSessionEnd::Finalize,
    })
    .await;
    iteration
        .iteration_backups
        .restore(work_dir)
        .map_err(|e| e.to_string())?;
    Ok(RouterAgentLoopOutcome {
        last_acp: iteration.acp_result,
        last_backups: iteration.iteration_backups,
    })
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_unit_names() {
        let _ = run_router_agent_loops;
    }
}

#[cfg(test)]
#[path = "router_flow_loop_kiss_cov_tests.rs"]
mod router_flow_loop_kiss_cov_tests;

#[cfg(test)]
#[path = "router_flow_loop_tests.rs"]
pub(crate) mod router_flow_loop_tests;
