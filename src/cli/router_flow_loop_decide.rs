use crate::agent_process::AcpJsonlTrace;
use crate::cli::router_flow::router_flow_acp::RouterExitSummarize;
use crate::cli::router_machine::{
    Completion, Effect, Iteration, RouterEvent, RouterState, transition,
};
use crate::cli::workflow_router_shared::run_router_workspace_gates;
use malvin::artifacts::{RunArtifacts, SessionDotfileBackups};

pub(crate) enum RouterLoopDecision {
    Continue,
    Exit,
    ExitGatesFailed(String),
}

pub(crate) struct RouterLoopExitInput<'a> {
    pub artifacts: &'a RunArtifacts,
    pub backups: &'a SessionDotfileBackups,
    pub completion: Completion,
    pub gates: bool,
    pub agent_loop: usize,
    pub max_loops: usize,
}

pub(crate) fn decide_router_loop_exit(input: RouterLoopExitInput<'_>) -> RouterLoopDecision {
    if matches!(input.completion, Completion::Impasse) {
        malvin::herdr::notify_working();
    }
    let iteration = Iteration::new(input.agent_loop, input.max_loops)
        .expect("router loop bounds are validated before iteration");
    let state = RouterState::Restoring {
        iteration,
        completion: input.completion,
    };
    let policy = transition(
        state.clone(),
        RouterEvent::FilesRestored {
            gates_enabled: input.gates,
        },
    )
    .expect("files restored is valid after a router iteration");
    let trace = AcpJsonlTrace::new(input.artifacts.run_dir.join("trace.jsonl"), "router".into());
    trace.append_line(
        "internal",
        &serde_json::json!({
            "event": "router_transition",
            "from": format!("{state:?}"),
            "to": format!("{:?}", policy.next),
        })
        .to_string(),
    );
    match policy.effect {
        Some(Effect::RunGates) => decide_router_gates_exit(
            input.artifacts,
            input.backups,
            input.agent_loop,
            input.max_loops,
        ),
        Some(Effect::CloseSession) => RouterLoopDecision::Continue,
        Some(Effect::SendSummary) => RouterLoopDecision::Exit,
        effect => panic!("unexpected router policy effect after restore: {effect:?}"),
    }
}

pub(crate) fn decide_router_gates_exit(
    artifacts: &RunArtifacts,
    backups: &SessionDotfileBackups,
    agent_loop: usize,
    max_loops: usize,
) -> RouterLoopDecision {
    malvin::gate_loop_session::set_active_gate_iteration(Some(agent_loop));
    let decision = match run_router_workspace_gates(artifacts, backups, true) {
        Ok(()) => RouterLoopDecision::Exit,
        Err(detail) if agent_loop == max_loops => RouterLoopDecision::ExitGatesFailed(detail),
        Err(_) => RouterLoopDecision::Continue,
    };
    malvin::gate_loop_session::set_active_gate_iteration(None);
    decision
}

pub(crate) fn prefer_exit_gates_over_acp(
    last_acp: &Result<(), String>,
    decision: RouterLoopDecision,
) -> RouterLoopDecision {
    match decision {
        RouterLoopDecision::ExitGatesFailed(detail) => RouterLoopDecision::ExitGatesFailed(detail),
        _ if last_acp.is_err() => RouterLoopDecision::Exit,
        RouterLoopDecision::Continue => RouterLoopDecision::Continue,
        RouterLoopDecision::Exit => RouterLoopDecision::Exit,
    }
}

pub(crate) const fn router_exit_summarize_for(
    decision: &RouterLoopDecision,
) -> RouterExitSummarize {
    match decision {
        RouterLoopDecision::Continue => RouterExitSummarize::Skip,
        RouterLoopDecision::Exit | RouterLoopDecision::ExitGatesFailed(_) => {
            RouterExitSummarize::Run
        }
    }
}
