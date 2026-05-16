use std::collections::HashMap;
use std::path::Path;

use super::constants::REVIEW_WRITE_FILE;
use super::{Orchestrator, WorkflowError, format_prompt_path};

pub async fn run_review_write_prompt(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    reviewers_subdir: &Path,
    attempt: usize,
) -> Result<(), WorkflowError> {
    let mut write_ctx = context.clone();
    write_ctx.insert(
        "reviewers_subdir".to_string(),
        format_prompt_path(reviewers_subdir, &orchestrator.artifacts.work_dir),
    );
    let suffix = format!("review_write_attempt_{attempt}");
    orchestrator
        .run_coder_prompt(
            REVIEW_WRITE_FILE,
            &write_ctx,
            &suffix,
            crate::run_timing::TimingPhase::ReviewWrite,
        )
        .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_stringify_review_fanout_write_units() {
        let _ = stringify!(super::run_review_write_prompt);
    }
}
