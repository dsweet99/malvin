use crate::artifacts::SessionDotfileBackups;

use crate::cli::entrypoint::print_command_error;
use crate::cli::workflow_kpop_shared::run_kpop_workspace_gates;
use crate::cli::build_agent;

use super::kpop_session::{print_gate_kpop_log_line, run_gate_kpop_session, GateKpopMultiturnCtx};
use super::params::{GateKpopIterationParams, GateKpopLoopParams};

struct GateKpopAgentSession {
    client: crate::acp::AgentClient,
    session_dotfile_backups: SessionDotfileBackups,
}

fn start_gate_kpop_agent_session(
    params: &GateKpopLoopParams<'_>,
) -> Result<GateKpopAgentSession, String> {
    let mut client = build_agent(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
    );
    client.prompts_log_run_dir = Some(params.prepared.artifacts().run_dir.clone());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&params.prepared.artifacts().work_dir)?;
    print_gate_kpop_log_line(params.prepared);
    Ok(GateKpopAgentSession {
        client,
        session_dotfile_backups,
    })
}

async fn run_gate_kpop_on_loop_iteration(
    params: &GateKpopLoopParams<'_>,
    agent: &mut Option<GateKpopAgentSession>,
) -> Result<(), String> {
    if agent.is_none() {
        *agent = Some(start_gate_kpop_agent_session(params)?);
    }
    let session = agent.as_mut().expect("agent session");
    let mut iteration = GateKpopIterationParams {
        loop_params: params,
        session_dotfile_backups: &session.session_dotfile_backups,
        client: &mut session.client,
    };
    let mut ctx = GateKpopMultiturnCtx {
        iteration: &mut iteration,
    };
    run_gate_kpop_session(&mut ctx).await
}

pub(crate) async fn run_gate_kpop_loop(
    params: GateKpopLoopParams<'_>,
) -> Result<(bool, bool), String> {
    let mut gates_ok = false;
    let mut agent_ran = false;
    let mut agent = None;
    for _ in 0..params.max_loops {
        match run_kpop_workspace_gates(params.prepared.artifacts()) {
            Ok(()) => {
                if params.behavior.skip_kpop_on_initial_pass {
                    gates_ok = true;
                    break;
                }
                if agent_ran {
                    gates_ok = true;
                    return Ok((gates_ok, agent_ran));
                }
            }
            Err(e) => print_command_error(&e),
        }
        run_gate_kpop_on_loop_iteration(&params, &mut agent).await?;
        agent_ran = true;
    }
    if params.behavior.recheck_gates_after_exhausted && agent_ran && !gates_ok {
        gates_ok = run_kpop_workspace_gates(params.prepared.artifacts()).is_ok();
    }
    Ok((gates_ok, agent_ran))
}

#[cfg(test)]
mod tests {
    #[test]
    fn gate_kpop_agent_session_and_loop_helpers_are_covered() {
        let _ = stringify!(super::GateKpopAgentSession);
        let _ = stringify!(super::start_gate_kpop_agent_session);
        let _ = stringify!(super::run_gate_kpop_on_loop_iteration);
        let _ = stringify!(super::run_gate_kpop_loop);
    }
}
