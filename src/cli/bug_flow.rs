use crate::artifacts::RunArtifacts;
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::prompts::{PromptError, PromptStore};

use super::SharedOpts;
use super::kpop_flow::{
    KpopAcpMultiturnCtx, KpopPrepared, KpopTurnPrompts, kpop_emit_startup, kpop_run_acp_multiturn,
    prepare_kpop_run,
};
use super::run_emit::emit_run_startup_sequence;
use super::{BugArgs, KpopArgs};
use super::{
    WorkflowCliOptions, build_agent, format_workspace_gate_failure, prepare_bug_prompt_store,
};
use crate::DEFAULT_LEARN_MIN_ELAPSED_MS as LEARN_MIN_ELAPSED_MS;
use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

const BUG_KPOP_REQUEST: &str = "Find a serious bug in this codebase.";

const BUG_FOLLOWUP_PLAN: &str = r"# Post-KPOP bug remediation

KPOP ended with `## KPOP_SOLVED`. The experiment log under `_kpop/` is the authoritative bug description.

Malvin will run two coder prompts in order: `bug_regression_test.md`, then `bug_fix.md`.
";

pub(crate) fn kpop_args_from_bug(bug: &BugArgs) -> KpopArgs {
    KpopArgs {
        max_hypotheses: bug.max_hypotheses,
        p_creative: bug.p_creative,
        no_learn: bug.no_learn,
        request: Some(BUG_KPOP_REQUEST.to_string()),
    }
}

struct BugKpopPhase<'a> {
    kpop: &'a KpopArgs,
    workflow: WorkflowCliOptions,
    store_kpop: &'a PromptStore,
    client: &'a mut crate::acp::AgentClient,
    shared: &'a SharedOpts,
}

async fn run_bug_kpop_multiturn(phase: BugKpopPhase<'_>) -> Result<KpopPrepared, String> {
    use crate::kpop_progression::KpopMultiturnState;
    let prepared = prepare_kpop_run(phase.kpop)?;
    phase.client.prompts_log_run_dir = Some(prepared.artifacts.run_dir.clone());
    super::error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));
    kpop_emit_startup(phase.kpop, phase.shared, &prepared.artifacts)?;
    let builder = crate::kpop_multiturn_prompts::KpopMultiturnPrompts::Turn(KpopTurnPrompts {
        store: phase.store_kpop,
        base: &prepared.context,
        request_text: &prepared.text,
        prepend_rules_once: true,
    });
    let mut state = KpopMultiturnState::new(
        builder,
        prepared.exp_log_path.clone(),
        phase.kpop.max_hypotheses,
        phase.kpop.p_creative,
    )?;
    let acp_result = kpop_run_acp_multiturn(KpopAcpMultiturnCtx {
        client: phase.client,
        prepared: &prepared,
        workflow: phase.workflow,
        state: &mut state,
        store: phase.store_kpop,
    })
    .await;
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_result,
        &prepared.artifacts.work_dir,
        &prepared.session_dotfile_backups,
        &prepared.artifacts.artifact_result_md(),
    )?;
    Ok(prepared)
}

fn ensure_kpop_solved(prepared: &KpopPrepared) -> Result<(), String> {
    use crate::kpop_progression::agent_declared_success;
    let exp_text = std::fs::read_to_string(&prepared.exp_log_path).map_err(|e| e.to_string())?;
    if agent_declared_success(&exp_text) {
        return Ok(());
    }
    Err(
        "KPOP did not record success: add a line exactly `## KPOP_SOLVED` to the experiment log once a serious bug is confirmed. Stopping before regression-test and fix coder phases.".to_string(),
    )
}

struct BugRunTail<'a> {
    bug: &'a BugArgs,
    shared: &'a SharedOpts,
    workflow: WorkflowCliOptions,
}

async fn finish_bug_after_kpop(
    tail: BugRunTail<'_>,
    prepared: KpopPrepared,
    client: &mut crate::acp::AgentClient,
) -> Result<(), String> {
    ensure_kpop_solved(&prepared)?;
    let artifacts = prepared.into_bug_followup_artifacts(BUG_FOLLOWUP_PLAN)?;
    super::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));

    if !tail.bug.skip_pre_checks {
        run_repo_workspace_gates(
            &artifacts.work_dir,
            RepoGateOutput::Tagged,
            Some(&artifacts.run_dir),
        )
        .map_err(|e| format_workspace_gate_failure("malvin bug", &e))?;
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

pub async fn run_bug(
    bug: BugArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    use crate::kpop_creative_enabled;
    let kpop = kpop_args_from_bug(&bug);
    let store_kpop =
        super::prepare_kpop_prompt_store(workflow, kpop_creative_enabled(kpop.p_creative))?;
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let mut client = build_agent(shared, workflow, emit_stdout_markdown);
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let r = async {
        let prepared = run_bug_kpop_multiturn(BugKpopPhase {
            kpop: &kpop,
            workflow,
            store_kpop: &store_kpop,
            client: &mut client,
            shared,
        })
        .await?;

        finish_bug_after_kpop(
            BugRunTail {
                bug: &bug,
                shared,
                workflow,
            },
            prepared,
            &mut client,
        )
        .await
    }
    .await;
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r
}

#[cfg(test)]
mod tests {
    use super::{BUG_KPOP_REQUEST, BugArgs, kpop_args_from_bug};

    #[test]
    fn kpop_args_from_bug_maps_bug_fields() {
        let bug = BugArgs {
            max_hypotheses: 7,
            p_creative: 0.25,
            no_learn: true,
            skip_pre_checks: true,
        };
        let kpop = kpop_args_from_bug(&bug);
        assert_eq!(kpop.max_hypotheses, 7);
        assert!((kpop.p_creative - 0.25).abs() < f64::EPSILON);
        assert!(kpop.no_learn);
        assert_eq!(kpop.request.as_deref(), Some(BUG_KPOP_REQUEST));
    }
}
