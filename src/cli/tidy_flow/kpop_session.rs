use crate::artifacts::SessionDotfileBackups;
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::kpop_progression::KpopMultiturnState;
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

use crate::cli::kpop_flow::{
    KpopAcpMultiturnCtx, KpopPrepared, KpopTurnPrompts, kpop_emit_startup, kpop_run_acp_multiturn,
};
use crate::cli::{
    format_workspace_gate_failure, KpopArgs, SharedOpts, WorkflowCliOptions,
};

use super::prep::write_checks_do_not_pass_for_artifacts;
use super::run_startup::TidyKpopPrepared;
use super::{effective_tidy_max_loops, TidyArgs};

pub(super) struct TidyKpopMultiturnRequest<'a> {
    pub tidy: &'a TidyArgs,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub client: &'a mut crate::acp::AgentClient,
    pub prepared: &'a TidyKpopPrepared,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
}

fn kpop_args_from_tidy(tidy: &TidyArgs, request: &str) -> KpopArgs {
    KpopArgs {
        max_hypotheses: effective_tidy_max_loops(tidy.max_loops),
        no_learn: tidy.no_learn,
        request: Some(request.to_string()),
    }
}

fn tidy_kpop_prepared(prepared: &TidyKpopPrepared, backups: SessionDotfileBackups) -> KpopPrepared {
    KpopPrepared {
        artifacts: prepared.artifacts.clone(),
        exp_log_path: prepared.exp_log_path.clone(),
        context: prepared.context.clone(),
        text: prepared.request_text.clone(),
        session_dotfile_backups: backups,
    }
}

async fn run_tidy_kpop_multiturn(req: &mut TidyKpopMultiturnRequest<'_>) -> Result<(), String> {
    let kpop = kpop_args_from_tidy(req.tidy, &req.prepared.request_text);
    kpop_emit_startup(&kpop, req.shared, &req.prepared.artifacts)?;
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
    let kpop_prepared = tidy_kpop_prepared(req.prepared, req.session_dotfile_backups.clone());
    kpop_run_acp_multiturn(KpopAcpMultiturnCtx {
        client: req.client,
        prepared: &kpop_prepared,
        workflow: req.workflow,
        state: &mut state,
        store: &req.prepared.store,
    })
    .await
}

pub(super) fn tidy_post_kpop_gates(prepared: &TidyKpopPrepared) -> Result<(), String> {
    if run_repo_workspace_gates(
        prepared.artifacts.work_dir.as_path(),
        RepoGateOutput::Tagged,
        Some(prepared.artifacts.run_dir.as_path()),
    )
    .is_ok()
    {
        return Ok(());
    }
    write_checks_do_not_pass_for_artifacts(&prepared.artifacts)?;
    Err(format_workspace_gate_failure(
        "malvin tidy",
        "workspace quality gates did not pass after the kpop session",
    ))
}

pub(super) fn print_tidy_kpop_log_line(prepared: &TidyKpopPrepared) {
    let kpop_id = crate::malvin_short_id();
    let log_line = crate::cli::bug_id_lookup_kpop::kpop_log_line(
        &kpop_id,
        &prepared.artifacts.work_dir,
        &prepared.artifacts.run_dir,
        &prepared.exp_log_path,
    );
    print_stdout_line(MALVIN_WHO, &log_line);
}

pub(super) async fn run_tidy_kpop_session(
    req: &mut TidyKpopMultiturnRequest<'_>,
) -> Result<(), String> {
    run_tidy_kpop_multiturn(req).await?;
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        Ok(()),
        &req.prepared.artifacts.work_dir,
        req.session_dotfile_backups,
        &req.prepared.artifacts.artifact_result_md(),
    )
}

pub(super) fn tidy_run_workspace_gates(prepared: &TidyKpopPrepared) -> Result<(), String> {
    run_repo_workspace_gates(
        prepared.artifacts.work_dir.as_path(),
        RepoGateOutput::Tagged,
        Some(prepared.artifacts.run_dir.as_path()),
    )
}

pub(super) fn tidy_finish_after_gates_pass(
    tidy: &TidyArgs,
    shared: &SharedOpts,
    prepared: &TidyKpopPrepared,
    agent_ran: bool,
) -> Result<(), String> {
    if !agent_ran {
        let kpop = kpop_args_from_tidy(tidy, &prepared.request_text);
        kpop_emit_startup(&kpop, shared, &prepared.artifacts)?;
    }
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

pub(super) fn tidy_fail_after_exhausted_loops(prepared: &TidyKpopPrepared) -> Result<(), String> {
    tidy_post_kpop_gates(prepared)
}
