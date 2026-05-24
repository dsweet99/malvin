use std::collections::HashMap;

use crate::run_timing::TimingPhase;

use super::review_loop::run_code_review_phase;
use super::{Orchestrator, WorkflowError, check_plan::run_check_plan};

pub(super) async fn run_coder_session_until_pre_summary(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    if !orchestrator.config.skip_check_plan {
        run_check_plan(orchestrator, context).await?;
    }

    if orchestrator.config.dry_run {
        return Ok(());
    }

    (orchestrator.progress_callback)("Implement");
    orchestrator
        .run_coder_prompt("implement.md", context, "main", TimingPhase::Implement)
        .await?;
    orchestrator.fail_on_abort_result()?;

    run_review_phases_until_pre_summary(orchestrator, context).await
}

pub(super) async fn run_bug_remediation_until_pre_summary(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    (orchestrator.progress_callback)("Bug regression test");
    orchestrator
        .run_coder_prompt(
            "bug_regression_test.md",
            context,
            "test",
            TimingPhase::Implement,
        )
        .await?;
    orchestrator.fail_on_abort_result()?;
    (orchestrator.progress_callback)("Bug fix");
    orchestrator
        .run_coder_prompt("bug_fix.md", context, "fix", TimingPhase::Implement)
        .await?;
    orchestrator.fail_on_abort_result()?;
    Ok(())
}

pub(super) async fn run_coder_session_summary_only(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    (orchestrator.progress_callback)("Summary");
    let summary_body = orchestrator
        .prompts
        .render("summary.md", context)
        .map_err(|e| WorkflowError(e.0))?;
    orchestrator
        .run_coder_prompt_body(summary_body, "summary.md", "summary", TimingPhase::Summary)
        .await?;
    orchestrator.fail_on_abort_result()?;
    Ok(())
}

async fn run_review_phases_until_pre_summary(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    run_code_review_phase(orchestrator, context).await?;

    if orchestrator.config.run_learn && orchestrator.should_run_learn() {
        (orchestrator.progress_callback)("Learn");
        orchestrator
            .run_coder_prompt("learn.md", context, "final", TimingPhase::Learn)
            .await?;
        orchestrator.fail_on_abort_result()?;
    }
    Ok(())
}

#[cfg(test)]
mod session_flow_smoke_tests {
    use super::{
        run_bug_remediation_until_pre_summary, run_coder_session_summary_only,
        run_coder_session_until_pre_summary, run_review_phases_until_pre_summary,
    };
    use crate::acp::AgentClient;
    use crate::artifacts::RunArtifacts;
    use crate::orchestrator::orchestrator_test_support::{
        empty_dotfile_backups, no_session_client, workflow_ctx_for_smoke,
    };
    use crate::orchestrator::{Orchestrator, WorkflowConfig};
    use crate::prompts::PromptStore;

    enum NoSessionStep {
        Summary,
        BugRemediation,
        CoderUntilPreSummary { skip_check_plan: bool },
        ReviewPhases,
    }

    fn mk_orchestrator<'a>(
        client: &'a mut AgentClient,
        store: &'a PromptStore,
        artifacts: &'a RunArtifacts,
        skip_check_plan: bool,
    ) -> Orchestrator<'a> {
        Orchestrator {
            client,
            prompts: store,
            artifacts,
            config: WorkflowConfig {
                max_loops: 1,
                run_learn: false,
                learn_min_elapsed_ms: 0,
                skip_check_plan,
                dry_run: false,
            },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: empty_dotfile_backups(),
        }
    }

    async fn assert_fails_without_coder_session(step: NoSessionStep, expect_err: &'static str) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts, store, ctx) = workflow_ctx_for_smoke(&tmp, "sf_smoke");
        let mut client = no_session_client();
        let skip = matches!(
            &step,
            NoSessionStep::CoderUntilPreSummary {
                skip_check_plan: true
            }
        );
        let mut orch = mk_orchestrator(&mut client, &store, &artifacts, skip);
        let err = match step {
            NoSessionStep::Summary => run_coder_session_summary_only(&mut orch, &ctx).await,
            NoSessionStep::BugRemediation => {
                run_bug_remediation_until_pre_summary(&mut orch, &ctx).await
            }
            NoSessionStep::CoderUntilPreSummary { .. } => {
                run_coder_session_until_pre_summary(&mut orch, &ctx).await
            }
            NoSessionStep::ReviewPhases => {
                run_review_phases_until_pre_summary(&mut orch, &ctx).await
            }
        }
        .expect_err(expect_err);
        assert!(
            err.0.contains("begin_coder_session"),
            "unexpected err: {}",
            err.0
        );
    }

    #[tokio::test]
    async fn summary_only_errors_when_coder_session_not_open() {
        assert_fails_without_coder_session(
            NoSessionStep::Summary,
            "expected prompt without session",
        )
        .await;
    }

    #[tokio::test]
    async fn bug_remediation_until_pre_summary_errors_when_coder_session_not_open() {
        assert_fails_without_coder_session(
            NoSessionStep::BugRemediation,
            "expected prompt without session",
        )
        .await;
    }

    #[tokio::test]
    async fn coder_session_until_pre_summary_errors_when_coder_session_not_open_skip_plan() {
        assert_fails_without_coder_session(
            NoSessionStep::CoderUntilPreSummary {
                skip_check_plan: true,
            },
            "expected implement without session",
        )
        .await;
    }

    #[tokio::test]
    async fn review_phases_errors_when_coder_session_not_open() {
        assert_fails_without_coder_session(
            NoSessionStep::ReviewPhases,
            "expected review prompt without session",
        )
        .await;
    }
}
