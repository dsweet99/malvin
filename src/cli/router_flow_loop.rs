use crate::agent_backend::AgentBackend;
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::format_workspace_gate_failure;
use crate::cli::workflow_kpop_shared::effective_max_loops;
use crate::cli::SharedOpts;
use crate::prompts::PromptStore;
use crate::run_timing::acp_post_run::RunTimingSessionEnd;
use super::router_flow_acp::{
    finalize_router_acp_iteration, run_router_acp_open_iteration, RouterAcpIterationInput,
    RouterAcpIterationOutcome,
};
use super::router_flow_acp::router_flow_acp_support::empty_iteration_backups;

#[path = "router_flow_loop_decide.rs"]
mod router_flow_loop_decide;
pub(crate) use router_flow_loop_decide::{
    decide_router_loop_exit, router_exit_summarize_for, RouterLoopDecision, RouterLoopExitInput,
};

#[cfg(test)]
pub(crate) use router_flow_loop_decide::{
    decide_router_gates_exit, decide_router_loop_exit_not_done,
};

pub(crate) struct RouterAgentLoopInput<'a> {
    pub client: &'a mut AgentBackend,
    pub artifacts: &'a RunArtifacts,
    pub prompt_store: &'a PromptStore,
    pub shared: &'a SharedOpts,
    pub max_loops: usize,
}

pub(crate) struct RouterAgentLoopOutcome {
    pub last_acp: Result<(), String>,
    pub last_backups: SessionDotfileBackups,
}

struct RouterLoopStepResult {
    last_acp: Result<(), String>,
    last_backups: SessionDotfileBackups,
    decision: Option<RouterLoopDecision>,
}

/// Outer agent lifetimes: `header.md` → `kpop_common.md` → `router_a.md` → optional `router_b.md`, then at most one
/// `router_summarize.md` when exiting the outer loop (on the final open session before teardown).
pub(crate) async fn run_router_agent_loops(
    mut input: RouterAgentLoopInput<'_>,
) -> Result<RouterAgentLoopOutcome, String> {
    let max_loops = effective_max_loops(input.max_loops);
    let mut last_acp = Ok(());
    let mut last_backups = empty_iteration_backups();
    for agent_loop in 1..=max_loops {
        let step = run_one_router_loop_step(&mut input, agent_loop, max_loops).await?;
        last_acp = step.last_acp;
        last_backups = step.last_backups;
        if last_acp.is_err() {
            break;
        }
        match step.decision {
            None | Some(RouterLoopDecision::Exit) => break,
            Some(RouterLoopDecision::Continue) => {}
            Some(RouterLoopDecision::ExitGatesFailed(detail)) => {
                return Err(format_workspace_gate_failure("malvin", &detail));
            }
        }
    }
    Ok(RouterAgentLoopOutcome {
        last_acp,
        last_backups,
    })
}

async fn run_one_router_loop_step(
    input: &mut RouterAgentLoopInput<'_>,
    agent_loop: usize,
    max_loops: usize,
) -> Result<RouterLoopStepResult, String> {
    let session_end = if agent_loop == max_loops {
        RunTimingSessionEnd::Finalize
    } else {
        RunTimingSessionEnd::AccumulateRun
    };
    let open = run_router_acp_open_iteration(RouterAcpIterationInput {
        client: input.client,
        artifacts: input.artifacts,
        prompt_store: input.prompt_store,
        shared: input.shared,
        agent_loop,
        session_end,
    })
    .await;
    finish_router_loop_step(input, (agent_loop, max_loops, session_end), open).await
}

async fn finish_router_loop_step(
    input: &mut RouterAgentLoopInput<'_>,
    ids: (usize, usize, RunTimingSessionEnd),
    open: RouterAcpIterationOutcome,
) -> Result<RouterLoopStepResult, String> {
    let (agent_loop, max_loops, session_end) = ids;
    let work_dir = input.artifacts.work_dir.as_path();
    open.iteration_backups
        .restore(work_dir)
        .map_err(|e| e.to_string())?;
    let last_backups = open.iteration_backups;
    if !open.session_alive {
        return Ok(RouterLoopStepResult {
            last_acp: open.acp_result,
            last_backups,
            decision: None,
        });
    }
    let decision = decide_router_loop_exit(RouterLoopExitInput {
        artifacts: input.artifacts,
        backups: &last_backups,
        done: open.done,
        gates: input.shared.gates,
        agent_loop,
        max_loops,
    });
    let timing = open.timing.expect("alive session carries timing");
    let last_acp = finalize_router_acp_iteration(
        &mut RouterAcpIterationInput {
            client: input.client,
            artifacts: input.artifacts,
            prompt_store: input.prompt_store,
            shared: input.shared,
            agent_loop,
            session_end,
        },
        timing,
        router_exit_summarize_for(&decision),
    )
    .await;
    Ok(RouterLoopStepResult {
        last_acp,
        last_backups,
        decision: Some(decision),
    })
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
