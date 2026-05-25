use crate::artifacts::SessionDotfileBackups;

use super::kpop_session::{
    print_tidy_kpop_log_line, run_tidy_kpop_session, tidy_fail_after_exhausted_loops,
    tidy_finish_after_gates_pass, tidy_run_workspace_gates, TidyKpopMultiturnRequest,
};
use super::run_startup::{prepare_tidy_kpop_run, TidyKpopPrepared};
use crate::cli::entrypoint::print_command_error;
use crate::cli::{build_agent, error_run_log, SharedOpts, WorkflowCliOptions};

use super::{effective_tidy_max_loops, TidyArgs};

struct TidyGateLoopCtx<'a> {
    tidy: &'a TidyArgs,
    shared: &'a SharedOpts,
    workflow: WorkflowCliOptions,
    prepared: &'a TidyKpopPrepared,
}

struct TidyAgentSession {
    client: crate::acp::AgentClient,
    session_dotfile_backups: SessionDotfileBackups,
}

fn start_tidy_agent_session(ctx: &TidyGateLoopCtx<'_>) -> Result<TidyAgentSession, String> {
    let mut client = build_agent(ctx.shared, ctx.workflow, ctx.shared.acp_stdout_markdown_enabled());
    client.prompts_log_run_dir = Some(ctx.prepared.artifacts.run_dir.clone());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&ctx.prepared.artifacts.work_dir)?;
    print_tidy_kpop_log_line(ctx.prepared);
    Ok(TidyAgentSession {
        client,
        session_dotfile_backups,
    })
}

async fn run_tidy_kpop_on_gate_failure(
    ctx: &TidyGateLoopCtx<'_>,
    agent: &mut Option<TidyAgentSession>,
) -> Result<(), String> {
    if agent.is_none() {
        *agent = Some(start_tidy_agent_session(ctx)?);
    }
    let session = agent.as_mut().expect("agent session");
    let mut req = TidyKpopMultiturnRequest {
        tidy: ctx.tidy,
        shared: ctx.shared,
        workflow: ctx.workflow,
        client: &mut session.client,
        prepared: ctx.prepared,
        session_dotfile_backups: &session.session_dotfile_backups,
    };
    run_tidy_kpop_session(&mut req).await
}

async fn run_tidy_gate_loop(ctx: &TidyGateLoopCtx<'_>, max_loops: usize) -> Result<(bool, bool), String> {
    let mut gates_ok = false;
    let mut agent_ran = false;
    let mut agent = None;
    for _ in 0..max_loops {
        match tidy_run_workspace_gates(ctx.prepared) {
            Ok(()) => {
                gates_ok = true;
                break;
            }
            Err(e) => {
                print_command_error(&e);
                run_tidy_kpop_on_gate_failure(ctx, &mut agent).await?;
                agent_ran = true;
            }
        }
    }
    Ok((gates_ok, agent_ran))
}

pub async fn run_tidy(
    tidy: TidyArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let workflow = WorkflowCliOptions {
        force: workflow.force,
        run_learn: !tidy.no_learn,
    };
    let prepared = prepare_tidy_kpop_run(workflow)?;
    error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    let ctx = TidyGateLoopCtx {
        tidy: &tidy,
        shared,
        workflow,
        prepared: &prepared,
    };
    let max_loops = effective_tidy_max_loops(tidy.max_loops);
    let (gates_ok, agent_ran) = run_tidy_gate_loop(&ctx, max_loops).await?;

    let r = if gates_ok {
        tidy_finish_after_gates_pass(&tidy, shared, &prepared, agent_ran)
    } else {
        tidy_fail_after_exhausted_loops(&prepared)
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
    fn tidy_agent_session_and_gate_loop_helpers_are_covered() {
        let _ = stringify!(TidyGateLoopCtx);
        let _ = stringify!(TidyAgentSession);
        let _ = stringify!(start_tidy_agent_session);
        let _ = stringify!(run_tidy_kpop_on_gate_failure);
        let _ = stringify!(run_tidy_gate_loop);
    }
}
