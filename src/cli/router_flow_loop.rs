use crate::agent_backend::AgentBackend;
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use super::router_flow_acp::{run_router_acp_iteration, RouterAcpIterationInput};
use super::router_flow_acp::router_flow_acp_support::empty_iteration_backups;
use crate::cli::format_workspace_gate_failure;
use crate::cli::workflow_kpop_shared::{effective_max_loops, run_kpop_workspace_gates};
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
    pub max_loops: usize,
}

pub(crate) struct RouterAgentLoopOutcome {
    pub last_acp: Result<(), String>,
    pub last_backups: SessionDotfileBackups,
}

/// Outer agent lifetimes: requirements → one multi-group `KPop` → optional work, up to
/// [`effective_max_loops`] sessions. With `--gates`, workspace gate failure continues the loop
/// even when `KPop` chat says no work remains.
pub(crate) async fn run_router_agent_loops(
    input: RouterAgentLoopInput<'_>,
) -> Result<RouterAgentLoopOutcome, String> {
    let RouterAgentLoopInput {
        client,
        artifacts,
        coder,
        prompt_store,
        shared,
        max_loops,
    } = input;
    let work_dir = artifacts.work_dir.as_path();
    let max_loops = effective_max_loops(max_loops);
    let mut last_acp = Ok(());
    let mut last_backups = empty_iteration_backups();

    for agent_loop in 1..=max_loops {
        let session_end = if agent_loop == max_loops {
            RunTimingSessionEnd::Finalize
        } else {
            RunTimingSessionEnd::AccumulateRun
        };
        let iteration = run_router_acp_iteration(RouterAcpIterationInput {
            client,
            artifacts,
            coder,
            prompt_store,
            shared,
            agent_loop,
            session_end,
        })
        .await;
        iteration
            .iteration_backups
            .restore(work_dir)
            .map_err(|e| e.to_string())?;
        last_acp = iteration.acp_result;
        last_backups = iteration.iteration_backups;
        if last_acp.is_err() {
            break;
        }

        if shared.gates {
            match run_kpop_workspace_gates(artifacts, &last_backups, true) {
                Ok(()) => break,
                Err(detail) if agent_loop == max_loops => {
                    return Err(format_workspace_gate_failure("malvin", &detail));
                }
                Err(_) => {
                    // Gate failure restarts even when chat said all_no_work.
                }
            }
        } else if iteration.all_no_work {
            break;
        }
        // Without gates: any residual work continues while budget remains; exhausted → success.
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

#[cfg(test)]
#[path = "router_flow_loop_gates_tests.rs"]
mod router_flow_loop_gates_tests;
