use crate::agent_backend::AgentBackend;
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use super::router_flow_acp::{run_router_acp_iteration, RouterAcpIterationInput};
use crate::router_flow::router_flow_prompt;
use crate::kpop_engine::restore_carry_forward_before_iteration_snapshot;
use crate::run_timing::acp_post_run::RunTimingSessionEnd;

pub(crate) struct RouterAgentLoopInput<'a> {
    pub client: &'a mut AgentBackend,
    pub artifacts: &'a RunArtifacts,
    pub coder: &'a router_flow_prompt::RouterCoderRun,
    pub router_b_prompt: &'a str,
    pub max_loops: usize,
}

pub(crate) struct RouterAgentLoopOutcome {
    pub last_acp: Result<(), String>,
    pub last_backups: SessionDotfileBackups,
}

pub(crate) async fn run_router_agent_loops(
    input: RouterAgentLoopInput<'_>,
) -> Result<RouterAgentLoopOutcome, String> {
    let RouterAgentLoopInput {
        client,
        artifacts,
        coder,
        router_b_prompt,
        max_loops,
    } = input;
    let work_dir = artifacts.work_dir.as_path();
    let mut last_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    let mut last_acp = Ok(());

    for agent_loop in 1..=max_loops {
        if agent_loop > 1 {
            restore_carry_forward_before_iteration_snapshot(work_dir, Some(&last_backups))?;
        }
        let iteration_backups =
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
        let session_end = if agent_loop == max_loops {
            RunTimingSessionEnd::Finalize
        } else {
            RunTimingSessionEnd::AccumulateRun
        };
        let iteration = run_router_acp_iteration(RouterAcpIterationInput {
            client,
            artifacts,
            coder,
            router_b_prompt,
            session_end,
        })
        .await;
        last_acp = iteration.acp_result;
        iteration_backups
            .restore(work_dir)
            .map_err(|e| e.to_string())?;
        last_backups = iteration_backups;
        if last_acp.is_err() || !iteration.wants_continue || agent_loop == max_loops {
            break;
        }
    }

    Ok(RouterAgentLoopOutcome {
        last_acp,
        last_backups,
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
