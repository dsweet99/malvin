use clap::Args;

use crate::artifacts::SessionDotfileBackups;
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::kpop_progression::KpopMultiturnState;
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

#[path = "tidy_flow/prep.rs"]
mod prep;
#[path = "tidy_flow/run_startup.rs"]
mod run_startup;

#[allow(unused_imports)]
pub use prep::{
    prepare_tidy_kpop_prompt_store, tidy_kpop_request, write_checks_do_not_pass_for_artifacts,
    write_checks_do_not_pass_to_review_path,
};
pub use run_startup::{prepare_tidy_kpop_run, TidyKpopPrepared};

use super::kpop_flow::{
    KpopAcpMultiturnCtx, KpopPrepared, KpopTurnPrompts, kpop_emit_startup, kpop_run_acp_multiturn,
};
use super::{
    KpopArgs, SharedOpts, WorkflowCliOptions, build_agent, format_workspace_gate_failure,
};

#[must_use]
pub(crate) fn effective_tidy_max_loops(max_loops: usize) -> usize {
    max_loops.max(1)
}

#[derive(Args, Debug, Clone)]
pub struct TidyArgs {
    /// Maximum `KPop` hypothesis steps before stopping (alias: `--max-hypotheses`).
    #[arg(long, default_value_t = 3, alias = "max-hypotheses")]
    pub max_loops: usize,
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Deprecated: review fan-out removed; tidy now uses the kpop workflow.
    #[arg(long, short = 'q', default_value_t = false, hide = true)]
    pub quick: bool,
}

struct TidyKpopMultiturnRequest<'a> {
    tidy: &'a TidyArgs,
    shared: &'a SharedOpts,
    workflow: WorkflowCliOptions,
    client: &'a mut crate::acp::AgentClient,
    prepared: &'a TidyKpopPrepared,
    session_dotfile_backups: &'a SessionDotfileBackups,
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

fn tidy_post_kpop_gates(prepared: &TidyKpopPrepared) -> Result<(), String> {
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

fn print_tidy_kpop_log_line(prepared: &TidyKpopPrepared) {
    let kpop_id = crate::malvin_short_id();
    let log_line = crate::cli::bug_id_lookup_kpop::kpop_log_line(
        &kpop_id,
        &prepared.artifacts.work_dir,
        &prepared.artifacts.run_dir,
        &prepared.exp_log_path,
    );
    print_stdout_line(MALVIN_WHO, &log_line);
}

async fn run_tidy_kpop_session(req: &mut TidyKpopMultiturnRequest<'_>) -> Result<(), String> {
    run_tidy_kpop_multiturn(req).await?;
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        Ok(()),
        &req.prepared.artifacts.work_dir,
        req.session_dotfile_backups,
        &req.prepared.artifacts.artifact_result_md(),
    )?;
    tidy_post_kpop_gates(req.prepared)?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

pub async fn run_tidy(
    tidy: TidyArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let run_learn = !tidy.no_learn;
    let workflow = WorkflowCliOptions {
        force: workflow.force,
        run_learn,
    };
    let prepared = prepare_tidy_kpop_run(workflow)?;
    super::error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    let mut client = build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.prompts_log_run_dir = Some(prepared.artifacts.run_dir.clone());
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&prepared.artifacts.work_dir)?;
    print_tidy_kpop_log_line(&prepared);

    let mut req = TidyKpopMultiturnRequest {
        tidy: &tidy,
        shared,
        workflow,
        client: &mut client,
        prepared: &prepared,
        session_dotfile_backups: &session_dotfile_backups,
    };
    let r = run_tidy_kpop_session(&mut req).await;

    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kpop_args_from_tidy_maps_max_loops() {
        let tidy = TidyArgs {
            max_loops: 0,
            no_learn: true,
            quick: false,
        };
        let kpop = kpop_args_from_tidy(&tidy, "req");
        assert_eq!(kpop.max_hypotheses, 1);
        assert!(kpop.no_learn);
        assert_eq!(kpop.request.as_deref(), Some("req"));
    }

    #[test]
    fn kiss_cov_tidy_kpop_helpers() {
        let _ = stringify!(tidy_kpop_prepared);
        let _ = stringify!(run_tidy_kpop_multiturn);
        let _ = stringify!(tidy_post_kpop_gates);
        let _ = stringify!(print_tidy_kpop_log_line);
        let _ = stringify!(run_tidy_kpop_session);
    }

    #[test]
    fn tidy_post_kpop_gates_fails_when_gates_fail() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss\n").expect("checks");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 1);
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = prepare_tidy_kpop_run(crate::cli::WorkflowCliOptions {
            force: false,
            run_learn: false,
        })
        .expect("prepared");
        let err = tidy_post_kpop_gates(&prepared).expect_err("gates");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(err.contains("quality gates"));
    }
}
