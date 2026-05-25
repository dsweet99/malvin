use crate::artifacts::SessionDotfileBackups;
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::kpop_progression::KpopMultiturnState;
use crate::output::{MALVIN_WHO, print_stdout_line};

use crate::cli::kpop_flow::{
    KpopAcpMultiturnCtx, KpopPrepared, KpopTurnPrompts, kpop_run_acp_multiturn,
};
use crate::cli::{
    SharedOpts, WorkflowCliOptions, KpopArgs,
};
use crate::cli::workflow_kpop_shared::{
    finish_kpop_acp_session, post_kpop_session_gates, print_kpop_session_log_line,
    run_kpop_workspace_gates,
};

use super::run_startup::CodeKpopPrepared;
use super::{effective_code_max_loops, CodeArgs};

pub(super) struct CodeKpopMultiturnRequest<'a> {
    pub code: &'a CodeArgs,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub client: &'a mut crate::acp::AgentClient,
    pub prepared: &'a CodeKpopPrepared,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
}

fn kpop_args_from_code(code: &CodeArgs, request: &str) -> KpopArgs {
    KpopArgs {
        max_hypotheses: effective_code_max_loops(code.max_loops),
        no_learn: code.no_learn,
        request: Some(request.to_string()),
    }
}

async fn run_code_kpop_multiturn(req: &mut CodeKpopMultiturnRequest<'_>) -> Result<(), String> {
    let kpop = kpop_args_from_code(req.code, &req.prepared.startup_request);
    crate::cli::run_emit::emit_run_startup_sequence(
        &req.prepared.artifacts,
        req.shared.tee_startup_stdout(),
        &req.prepared.startup_request,
    )?;
    let builder = KpopMultiturnPrompts::Turn(KpopTurnPrompts {
        store: &req.prepared.store,
        base: &req.prepared.context,
        request_text: &req.prepared.request_text,
        prepend_rules_once: true,
    });
    let mut state = KpopMultiturnState::new(
        builder,
        req.prepared.exp_log_path.clone(),
        kpop.max_hypotheses,
        0.0,
    )?;
    let kpop_prepared = KpopPrepared {
        artifacts: req.prepared.artifacts.clone(),
        exp_log_path: req.prepared.exp_log_path.clone(),
        context: req.prepared.context.clone(),
        text: req.prepared.request_text.clone(),
        session_dotfile_backups: req.session_dotfile_backups.clone(),
    };
    kpop_run_acp_multiturn(KpopAcpMultiturnCtx {
        client: req.client,
        prepared: &kpop_prepared,
        workflow: req.workflow,
        state: &mut state,
        store: &req.prepared.store,
    })
    .await
}

pub(super) fn code_post_kpop_gates(prepared: &CodeKpopPrepared) -> Result<(), String> {
    post_kpop_session_gates("malvin code", &prepared.artifacts)
}

pub(super) fn print_code_kpop_log_line(prepared: &CodeKpopPrepared) {
    print_kpop_session_log_line(&prepared.artifacts, &prepared.exp_log_path);
}

pub(super) async fn run_code_kpop_session(
    req: &mut CodeKpopMultiturnRequest<'_>,
) -> Result<(), String> {
    run_code_kpop_multiturn(req).await?;
    finish_kpop_acp_session(&req.prepared.artifacts, req.session_dotfile_backups).await
}

pub(super) fn code_run_workspace_gates(prepared: &CodeKpopPrepared) -> Result<(), String> {
    run_kpop_workspace_gates(&prepared.artifacts)
}

pub(super) fn code_finish_after_gates_pass(
    code: &CodeArgs,
    shared: &SharedOpts,
    prepared: &CodeKpopPrepared,
    agent_ran: bool,
) -> Result<(), String> {
    if !agent_ran {
        crate::cli::run_emit::emit_run_startup_sequence(
            &prepared.artifacts,
            shared.tee_startup_stdout(),
            &prepared.startup_request,
        )?;
    }
    let _ = code;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

pub(super) fn code_fail_after_exhausted_loops(prepared: &CodeKpopPrepared) -> Result<(), String> {
    code_post_kpop_gates(prepared)
}
