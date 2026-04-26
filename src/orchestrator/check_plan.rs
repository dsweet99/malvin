use crate::review_sync::is_lgtm_str;
use crate::run_timing::TimingPhase;
use std::collections::HashMap;
use std::path::Path;

use super::Orchestrator;
use super::WorkflowError;
use super::clear_review_file;

pub(super) async fn run_check_plan(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let review_path = orchestrator.artifacts.artifact_review_md();
    for attempt in 0..super::CHECK_PLAN_MAX_ATTEMPTS {
        if attempt > 0 {
            (orchestrator.progress_callback)(
                "CheckPlan: agent did not write review file, retrying",
            );
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        let Some(contents) = run_check_plan_attempt(orchestrator, context, &review_path).await? else {
            continue;
        };
        if is_lgtm_str(&contents) {
            return orchestrator.finish_check_plan_after_lgtm();
        }
        (orchestrator.progress_callback)(&format!("Plan check failed:\n{contents}"));
        return Err(WorkflowError("check_plan did not pass".to_string()));
    }
    Err(WorkflowError(
        "check_plan: agent did not write review file after retries".to_string(),
    ))
}

async fn run_check_plan_attempt(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    review_path: &Path,
) -> Result<Option<String>, WorkflowError> {
    clear_review_file(review_path).map_err(|e| WorkflowError(format!("failed to clear review file: {e}")))?;
    (orchestrator.progress_callback)("CheckPlan");
    orchestrator
        .run_coder_prompt("check_plan.md", context, "check", TimingPhase::CheckPlan)
        .await?;
    read_check_plan_review_file(review_path)
}

fn read_check_plan_review_file(review_path: &Path) -> Result<Option<String>, WorkflowError> {
    if !review_path.exists() {
        return Ok(None);
    }
    Ok(Some(
        std::fs::read_to_string(review_path)
            .map_err(|e| {
                WorkflowError(format!(
                    "failed to read review file: {}: {e}",
                    review_path.display()
                ))
            })?,
    ))
}

#[cfg(test)]
mod tests {
    use super::read_check_plan_review_file;
    #[test]
    fn read_check_plan_review_file_returns_none_when_missing() {
        let t = tempfile::tempdir().unwrap();
        let review_path = t.path().join("missing.md");
        assert!(read_check_plan_review_file(&review_path).unwrap().is_none());
    }

    #[test]
    fn read_check_plan_review_file_returns_error_when_path_is_directory() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("unsupported-file");
        std::fs::create_dir(&dir).unwrap();
        let review_path = dir;
        let err = read_check_plan_review_file(&review_path).unwrap_err();
        assert!(err.0.contains("failed to read review file"));
    }
}
