use crate::artifacts::RunArtifacts;
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::prompts::{PromptError, PromptStore};

use super::bug_id_lookup::{ensure_exp_log_solved, BugIdResolved};
use super::run_emit::emit_run_startup_sequence;
use super::{BugArgs, SharedOpts, WorkflowCliOptions};
use super::{format_workspace_gate_failure, prepare_bug_prompt_store};
use crate::DEFAULT_LEARN_MIN_ELAPSED_MS as LEARN_MIN_ELAPSED_MS;
use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

pub(super) const BUG_FOLLOWUP_PLAN: &str = r"# Post-KPOP bug remediation

KPOP ended with `## KPOP_SOLVED`. The experiment log under `_kpop/` is the authoritative bug description.

Malvin will run two coder prompts in order: `bug_regression_test.md`, then `bug_fix.md`.
";

pub(super) const BUG_KPOP_REQUEST: &str = "Find a serious bug in this codebase.";

pub(super) struct BugRunTail<'a> {
    pub bug: &'a BugArgs,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub gate_retry_command: String,
}

pub(super) fn gate_retry_command(bug: &BugArgs) -> String {
    bug.bug_id.as_ref().map_or_else(
        || "malvin hunt --fix".to_string(),
        |id| format!("malvin hunt {id}"),
    )
}

pub(super) async fn finish_bug_remediation(
    tail: BugRunTail<'_>,
    artifacts: RunArtifacts,
    client: &mut crate::acp::AgentClient,
    skip_workspace_gates: bool,
) -> Result<(), String> {
    super::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    #[cfg(not(test))]
    crate::stdout_log_path::set_stdout_log_path(Some(artifacts.stdout_log_path()));

    if !skip_workspace_gates && !tail.bug.skip_pre_checks {
        run_repo_workspace_gates(
            &artifacts.work_dir,
            RepoGateOutput::Tagged,
            Some(&artifacts.run_dir),
        )
        .map_err(|e| format_workspace_gate_failure(&tail.gate_retry_command, &e))?;
    }

    let store = prepare_bug_prompt_store(tail.workflow)?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    emit_run_startup_sequence(
        &artifacts,
        tail.shared.tee_startup_stdout(),
        BUG_KPOP_REQUEST,
    )?;
    run_bug_remediation_orchestrator(client, &artifacts, &store, tail.workflow).await?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

pub(super) fn artifacts_for_fix_by_id(resolved: &BugIdResolved) -> Result<RunArtifacts, String> {
    let plan_path = resolved.run_dir.join("plan.md");
    if !plan_path.is_file() {
        std::fs::write(&plan_path, BUG_FOLLOWUP_PLAN).map_err(|e| e.to_string())?;
    }
    Ok(RunArtifacts {
        run_dir: resolved.run_dir.clone(),
        plan_path,
        work_dir: resolved.work_dir.clone(),
    })
}

pub(super) async fn run_bug_fix_by_id(
    tail: BugRunTail<'_>,
    resolved: BugIdResolved,
    client: &mut crate::acp::AgentClient,
) -> Result<(), String> {
    ensure_exp_log_solved(&resolved.exp_log_path)?;
    let artifacts = artifacts_for_fix_by_id(&resolved)?;
    finish_bug_remediation(tail, artifacts, client, false).await
}

async fn run_bug_remediation_orchestrator(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    store: &PromptStore,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    use crate::orchestrator::{Orchestrator, WorkflowConfig, WorkflowError, workflow_context};
    let session_dotfile_backups =
        crate::artifacts::SessionDotfileBackups::snapshot(&artifacts.work_dir)?;
    let ctx = workflow_context(artifacts, store, "bug").map_err(|e: PromptError| e.0)?;
    let mut orch = Orchestrator {
        client,
        prompts: store,
        artifacts,
        config: WorkflowConfig {
            max_loops: 5,
            run_learn: workflow.run_learn,
            learn_min_elapsed_ms: LEARN_MIN_ELAPSED_MS,
            skip_check_plan: true,
        },
        progress_callback: Box::new(|msg: &str| {
            print_stdout_line(MALVIN_WHO, msg);
        }),
        session_dotfile_backups: session_dotfile_backups.clone(),
    };
    let workflow_res = orch
        .run_bug_remediation_gap(&ctx, crate::orchestrator::mid_noop)
        .await
        .map_err(|e: WorkflowError| e.0);
    crate::acp_post_run::merge_acp_with_workspace_session_restore(
        workflow_res,
        &artifacts.work_dir,
        &session_dotfile_backups,
    )
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{finish_bug_remediation, run_bug_fix_by_id};

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = finish_bug_remediation;
        let _ = run_bug_fix_by_id;
    }
}
