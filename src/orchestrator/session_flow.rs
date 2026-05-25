use std::collections::HashMap;

use crate::run_timing::TimingPhase;

use super::{Orchestrator, WorkflowError};

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

#[cfg(test)]
mod session_flow_smoke_tests {
    use super::{run_bug_remediation_until_pre_summary, run_coder_session_summary_only};
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
    }

    fn mk_orchestrator<'a>(
        client: &'a mut AgentClient,
        store: &'a PromptStore,
        artifacts: &'a RunArtifacts,
    ) -> Orchestrator<'a> {
        Orchestrator {
            client,
            prompts: store,
            artifacts,
            config: WorkflowConfig {
                max_loops: 1,
                run_learn: false,
                learn_min_elapsed_ms: 0,
            },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: empty_dotfile_backups(),
        }
    }

    async fn assert_fails_without_coder_session(step: NoSessionStep, expect_err: &'static str) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts, store, ctx) = workflow_ctx_for_smoke(&tmp, "sf_smoke");
        let mut client = no_session_client();
        let mut orch = mk_orchestrator(&mut client, &store, &artifacts);
        let err = match step {
            NoSessionStep::Summary => run_coder_session_summary_only(&mut orch, &ctx).await,
            NoSessionStep::BugRemediation => {
                run_bug_remediation_until_pre_summary(&mut orch, &ctx).await
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
}
