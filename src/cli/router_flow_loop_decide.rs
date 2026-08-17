use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::workflow_kpop_shared::run_kpop_workspace_gates;
use crate::router_flow::router_flow_acp::RouterExitSummarize;


pub(crate) enum RouterLoopDecision {
    Continue,
    Exit,
    ExitGatesFailed(String),
}

pub(crate) struct RouterLoopExitInput<'a> {
    pub artifacts: &'a RunArtifacts,
    pub backups: &'a SessionDotfileBackups,
    pub done: bool,
    pub gates: bool,
    pub agent_loop: usize,
    pub max_loops: usize,
}

pub(crate) fn decide_router_loop_exit(input: RouterLoopExitInput<'_>) -> RouterLoopDecision {
    if input.done {
        if input.gates {
            return decide_router_gates_exit(
                input.artifacts,
                input.backups,
                input.agent_loop,
                input.max_loops,
            );
        }
        return RouterLoopDecision::Exit;
    }
    decide_router_loop_exit_not_done(input.agent_loop, input.max_loops)
}

pub(crate) fn decide_router_gates_exit(
    artifacts: &RunArtifacts,
    backups: &SessionDotfileBackups,
    agent_loop: usize,
    max_loops: usize,
) -> RouterLoopDecision {
    crate::gate_loop_session::set_active_gate_iteration(Some(agent_loop));
    let decision = match run_kpop_workspace_gates(artifacts, backups, true) {
        Ok(()) => RouterLoopDecision::Exit,
        Err(detail) if agent_loop == max_loops => RouterLoopDecision::ExitGatesFailed(detail),
        Err(_) => RouterLoopDecision::Continue,
    };
    crate::gate_loop_session::set_active_gate_iteration(None);
    decision
}

pub(crate) const fn decide_router_loop_exit_not_done(
    agent_loop: usize,
    max_loops: usize,
) -> RouterLoopDecision {
    if agent_loop == max_loops {
        RouterLoopDecision::Exit
    } else {
        RouterLoopDecision::Continue
    }
}

pub(crate) const fn router_exit_summarize_for(decision: &RouterLoopDecision) -> RouterExitSummarize {
    match decision {
        RouterLoopDecision::Continue => RouterExitSummarize::Skip,
        RouterLoopDecision::Exit | RouterLoopDecision::ExitGatesFailed(_) => RouterExitSummarize::Run,
    }
}
