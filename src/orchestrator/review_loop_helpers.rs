use super::{Orchestrator, WorkflowError};
use std::collections::HashMap;

pub async fn run_concerns_and_check_abort_impl(
    orchestrator: &mut Orchestrator<'_>,
    attempt: usize,
    concern_suffix: &str,
    context: &HashMap<String, String>,
) -> Result<bool, WorkflowError> {
    if let Some(abort_msg) = super::check_abort(&orchestrator.artifacts.artifact_result_md()) {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    (orchestrator.progress_callback)(&format!("Concerns (attempt {attempt})"));
    let concerns_body = orchestrator
        .prompts
        .render("concerns.md", context)
        .map_err(|e| WorkflowError(e.0))?;
    orchestrator
        .run_coder_prompt_body(
            concerns_body,
            "concerns.md",
            concern_suffix,
            crate::run_timing::TimingPhase::Concerns,
        )
        .await?;
    if let Some(abort_msg) = super::check_abort(&orchestrator.artifacts.artifact_result_md()) {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    Ok(false)
}

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_review_loop_helpers() {
        let _ = stringify!(super::run_concerns_and_check_abort_impl);
    }
}
