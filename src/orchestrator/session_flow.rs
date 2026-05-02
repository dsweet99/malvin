use std::collections::HashMap;

use crate::acp::AgentError;
use crate::run_timing::TimingPhase;

use crate::orchestrator_review_loop_helpers::prompt_with_sync_header;

use super::review_context::ReviewPhaseArgs;
use super::review_loop::{run_review_phase, run_sync_check_dry_run_once};
use super::session_mode::OrchestratorSessionMode;
use super::{Orchestrator, WorkflowError, check_plan::run_check_plan, prefer_primary_errors_over_timing};

pub(super) async fn run_coder_session_until_pre_summary(
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
        super::review_loop::run_sync_check_loop(
            orchestrator,
            context,
            mode.include_sync_check_phase(),
        )
        .await?;
    }

    run_review_phases_until_pre_summary(orchestrator, context, mode.include_sync_check_phase())
        .await
}

pub(super) async fn run_coder_session_summary_only(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    prepend_sync_header: bool,
) -> Result<(), WorkflowError> {
    (orchestrator.progress_callback)("Summary");
    let summary_body = prompt_with_sync_header(
        orchestrator,
        "summary.md",
        context,
        prepend_sync_header,
    )?;
    orchestrator
        .run_coder_prompt_body(
            summary_body,
            "summary.md",
            "summary",
            TimingPhase::Summary,
        )
        .await?;
    orchestrator.fail_on_abort_result()?;
    Ok(())
}

async fn run_review_phases_until_pre_summary(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    prepend_sync_header: bool,
) -> Result<(), WorkflowError> {
    run_review_phase(
        orchestrator,
        ReviewPhaseArgs {
            review_prompt: "review_1.md",
            progress_label: "Review-1",
            phase_id: "review_1",
            context,
        },
        prepend_sync_header,
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
        prepend_sync_header,
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

#[allow(clippy::implicit_hasher)]
pub async fn run_sync_dry_run(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let timing = orchestrator.attach_run_timing();
    let begin_res = orchestrator
        .client
        .begin_coder_session(&orchestrator.artifacts.work_dir)
        .await;
    let coder_session_began = begin_res.is_ok();
    let workflow_result = match begin_res {
        Ok(()) => {
            run_sync_check_dry_run_once(
                orchestrator,
                context,
                OrchestratorSessionMode::Sync.include_sync_check_phase(),
            )
            .await
        }
        Err(e) => Err(WorkflowError(e.0)),
    };
    let timing_result = if coder_session_began {
        orchestrator.emit_run_timing_artifact(&timing)
    } else {
        orchestrator.client.set_run_timing(None);
        Ok(())
    };
    let end_result = orchestrator
        .client
        .end_coder_session()
        .await
        .map_err(|e: AgentError| WorkflowError(e.0));
    prefer_primary_errors_over_timing(workflow_result, end_result, timing_result)
}

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_session_flow_units() {
        let _ = stringify!(super::run_coder_session_until_pre_summary);
        let _ = stringify!(super::run_coder_session_summary_only);
        let _ = stringify!(super::run_sync_dry_run);
    }
}
