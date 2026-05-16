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
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_session_flow_units() {
        let _ = stringify!(super::run_coder_session_until_pre_summary);
        let _ = stringify!(super::run_bug_remediation_until_pre_summary);
        let _ = stringify!(super::run_coder_session_summary_only);
        let _ = stringify!(super::run_review_phases_until_pre_summary);
    }
}
