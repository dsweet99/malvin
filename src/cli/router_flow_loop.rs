use super::router_flow_acp::router_flow_acp_support::empty_iteration_backups;
use super::router_flow_acp::{
    finalize_router_acp_iteration, run_router_acp_open_iteration, RouterAcpIterationInput,
    RouterAcpIterationOutcome,
};
use crate::cli::format_workspace_gate_failure;
use crate::cli::workflow_router_shared::effective_max_loops;
use crate::cli::{RouterOpts, SharedOpts};
use malvin::agent_backend::SdkClient;
use malvin::artifacts::{merge_and_sanitize_for_gate_restore, RunArtifacts, SessionDotfileBackups};
use malvin::prompts::PromptStore;
use malvin::run_timing::acp_post_run::RunTimingSessionEnd;
use std::path::Path;

#[path = "router_flow_loop_decide.rs"]
mod router_flow_loop_decide;
pub(crate) use router_flow_loop_decide::{
    decide_router_loop_exit, router_exit_summarize_for, RouterLoopDecision, RouterLoopExitInput,
};

pub(crate) struct RouterAgentLoopInput<'a> {
    pub client: &'a mut SdkClient,
    pub artifacts: &'a RunArtifacts,
    pub prompt_store: &'a PromptStore,
    pub shared: &'a SharedOpts,
    pub router: &'a RouterOpts,
    pub max_loops: usize,
    pub max_hypotheses: usize,
    pub watch_source: Option<&'a Path>,
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

pub(crate) fn restore_router_iteration_dotfiles(
    work_dir: &Path,
    iteration_anchor: &SessionDotfileBackups,
) -> Result<SessionDotfileBackups, String> {
    let progress =
        SessionDotfileBackups::snapshot(work_dir).unwrap_or_else(|_| iteration_anchor.clone());
    let merged = merge_and_sanitize_for_gate_restore(iteration_anchor, &progress, work_dir);
    merged.restore(work_dir)?;
    Ok(merged)
}

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
        match router_flow_loop_decide::prefer_exit_gates_over_acp(&last_acp, step.decision) {
            RouterLoopDecision::Continue => {}
            RouterLoopDecision::Exit => break,
            RouterLoopDecision::ExitGatesFailed(detail) => {
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
    malvin::artifacts::maybe_refresh_watched_plan(
        input.router.watch,
        input.watch_source,
        input.artifacts.plan_path.as_path(),
    )?;
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
        router: input.router,
        agent_loop,
        session_end,
        max_hypotheses: input.max_hypotheses,
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
    let (acp_result, iteration_backups, open_session) = match open {
        RouterAcpIterationOutcome::Closed {
            acp_result,
            iteration_backups,
        } => (acp_result, iteration_backups, None),
        RouterAcpIterationOutcome::Open {
            iteration_backups,
            done,
            timing,
        } => (Ok(()), iteration_backups, Some((done, timing))),
    };
    let last_backups = restore_router_iteration_dotfiles(work_dir, &iteration_backups)?;
    let Some((done, timing)) = open_session else {
        return Ok(RouterLoopStepResult {
            last_acp: acp_result,
            last_backups,
            decision: None,
        });
    };
    let decision = decide_router_loop_exit(RouterLoopExitInput {
        artifacts: input.artifacts,
        backups: &last_backups,
        done,
        gates: input.router.gates,
        agent_loop,
        max_loops,
    });
    let last_acp = finalize_router_acp_iteration(
        &mut RouterAcpIterationInput {
            client: input.client,
            artifacts: input.artifacts,
            prompt_store: input.prompt_store,
            shared: input.shared,
            router: input.router,
            agent_loop,
            session_end,
            max_hypotheses: input.max_hypotheses,
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
#[path = "router_flow_loop_tests.rs"]
mod router_flow_loop_tests;
