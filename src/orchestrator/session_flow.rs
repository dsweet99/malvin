use std::collections::HashMap;

use crate::run_timing::TimingPhase;

use super::{check_plan::run_check_plan, Orchestrator, WorkflowError};
use super::review_loop::run_review_phase;
use super::review_context::ReviewPhaseArgs;
use super::session_mode::OrchestratorSessionMode;

pub(super) async fn run_coder_session(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    mode: OrchestratorSessionMode,
) -> Result<(), WorkflowError> {
    if mode.include_implement_phase() && !orchestrator.config.skip_check_plan {
        run_check_plan(orchestrator, context).await?;
    }

    if mode.include_implement_phase() {
        (orchestrator.progress_callback)("Implement");
        orchestrator
            .run_coder_prompt("implement.md", context, "main", TimingPhase::Implement)
            .await?;
        orchestrator.fail_on_abort_result()?;
    }

    if mode.include_sync_check_phase() {
        super::review_loop::run_sync_check_loop(orchestrator, context).await?;
    }

    run_review_loop(orchestrator, context).await
}

async fn run_review_loop(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    run_review_phase(
        orchestrator,
        ReviewPhaseArgs {
            review_prompt: "review_1.md",
            progress_label: "Review-1",
            phase_id: "review_1",
            context,
        },
    )
    .await?;

    run_review_phase(
        orchestrator,
        ReviewPhaseArgs {
            review_prompt: "review_2.md",
            progress_label: "Review-2",
            phase_id: "review_2",
            context,
        },
    )
    .await?;

    if orchestrator.config.run_learn && orchestrator.should_run_learn() {
        (orchestrator.progress_callback)("Learn");
        orchestrator
            .run_coder_prompt("learn.md", context, "final", TimingPhase::Learn)
            .await?;
        orchestrator.fail_on_abort_result()?;
    }
    Ok(())
}
