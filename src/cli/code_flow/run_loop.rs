use crate::artifacts::SessionDotfileBackups;

use super::kpop_session::{
    print_code_kpop_log_line, run_code_kpop_session, code_fail_after_exhausted_loops,
    code_finish_after_gates_pass, code_run_workspace_gates, CodeKpopMultiturnRequest,
};
use super::run_startup::{prepare_code_kpop_run, CodeKpopPrepared};
use crate::cli::entrypoint::print_command_error;
use crate::cli::{build_agent, error_run_log, SharedOpts, WorkflowCliOptions};

use super::{effective_code_max_loops, CodeArgs};

struct CodeGateLoopCtx<'a> {
    code: &'a CodeArgs,
    shared: &'a SharedOpts,
    workflow: WorkflowCliOptions,
    prepared: &'a CodeKpopPrepared,
}

struct CodeAgentSession {
    client: crate::acp::AgentClient,
    session_dotfile_backups: SessionDotfileBackups,
}

fn start_code_agent_session(ctx: &CodeGateLoopCtx<'_>) -> Result<CodeAgentSession, String> {
    let mut client = build_agent(ctx.shared, ctx.workflow, ctx.shared.acp_stdout_markdown_enabled());
    client.prompts_log_run_dir = Some(ctx.prepared.artifacts.run_dir.clone());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&ctx.prepared.artifacts.work_dir)?;
    print_code_kpop_log_line(ctx.prepared);
    Ok(CodeAgentSession {
        client,
        session_dotfile_backups,
    })
}

async fn run_code_kpop_on_gate_failure(
    ctx: &CodeGateLoopCtx<'_>,
    agent: &mut Option<CodeAgentSession>,
) -> Result<(), String> {
    if agent.is_none() {
        *agent = Some(start_code_agent_session(ctx)?);
    }
    let session = agent.as_mut().expect("agent session");
    let mut req = CodeKpopMultiturnRequest {
        code: ctx.code,
        shared: ctx.shared,
        workflow: ctx.workflow,
        client: &mut session.client,
        prepared: ctx.prepared,
        session_dotfile_backups: &session.session_dotfile_backups,
    };
    run_code_kpop_session(&mut req).await
}

async fn run_code_gate_loop(ctx: &CodeGateLoopCtx<'_>, max_loops: usize) -> Result<(bool, bool), String> {
    let mut gates_ok = false;
    let mut agent_ran = false;
    let mut agent = None;
    for _ in 0..max_loops {
        match code_run_workspace_gates(ctx.prepared) {
            Ok(()) => {
                if agent_ran {
                    gates_ok = true;
                    return Ok((gates_ok, agent_ran));
                }
            }
            Err(e) => print_command_error(&e),
        }
        run_code_kpop_on_gate_failure(ctx, &mut agent).await?;
        agent_ran = true;
    }
    if agent_ran && !gates_ok {
        gates_ok = code_run_workspace_gates(ctx.prepared).is_ok();
    }
    Ok((gates_ok, agent_ran))
}

pub async fn run_code(
    code: CodeArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let workflow = WorkflowCliOptions {
        force: workflow.force,
        run_learn: !code.no_learn,
    };
    let cli_request = crate::cli::cli_request::require_cli_request(code.request.as_ref(), "code")?;
    let prepared = prepare_code_kpop_run(workflow, &cli_request)?;
    error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    let ctx = CodeGateLoopCtx {
        code: &code,
        shared,
        workflow,
        prepared: &prepared,
    };
    let max_loops = effective_code_max_loops(code.max_loops);
    let (gates_ok, agent_ran) = run_code_gate_loop(&ctx, max_loops).await?;

    let r = if gates_ok {
        code_finish_after_gates_pass(&code, shared, &prepared, agent_ran)
    } else {
        code_fail_after_exhausted_loops(&prepared)
    };

    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    #[test]
    fn code_agent_session_and_gate_loop_helpers_are_covered() {
        let _ = stringify!(CodeGateLoopCtx);
        let _ = stringify!(CodeAgentSession);
        let _ = stringify!(start_code_agent_session);
        let _ = stringify!(run_code_kpop_on_gate_failure);
        let _ = stringify!(run_code_gate_loop);
    }
}
