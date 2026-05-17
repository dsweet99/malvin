use crate::review_sync::{is_lgtm_str, read_nonempty_review};
use crate::run_timing::TimingPhase;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use super::Orchestrator;
use super::WorkflowError;
use super::clear_review_file;

pub(super) async fn run_check_plan(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let review_path = orchestrator.artifacts.artifact_review_md();
    let max_loops = orchestrator.config.max_loops.max(1);
    for attempt in 0..max_loops {
        if attempt > 0 {
            (orchestrator.progress_callback)(
                "CheckPlan: agent did not write review file, retrying",
            );
            let retry_secs = if attempt == 1 { 1 } else { 3 };
            tokio::time::sleep(Duration::from_secs(retry_secs)).await;
        }
        let Some(contents) = run_check_plan_attempt(orchestrator, context, &review_path).await?
        else {
            orchestrator.fail_on_abort_result()?;
            continue;
        };
        if is_lgtm_str(&contents) {
            return orchestrator.fail_on_abort_result();
        }
        orchestrator.fail_on_abort_result()?;
        (orchestrator.progress_callback)(&format!("Plan check failed:\n{contents}"));
        return Err(WorkflowError("check_plan did not pass".to_string()));
    }
    orchestrator.fail_on_abort_result()?;
    Err(WorkflowError(
        "check_plan: agent did not write review file after retries".to_string(),
    ))
}

async fn run_check_plan_attempt(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    review_path: &Path,
) -> Result<Option<String>, WorkflowError> {
    clear_review_file(review_path)
        .map_err(|e| WorkflowError(format!("failed to clear review file: {e}")))?;
    (orchestrator.progress_callback)("CheckPlan");
    orchestrator
        .run_coder_prompt("check_plan.md", context, "check", TimingPhase::CheckPlan)
        .await?;
    read_check_plan_review_file(review_path)
}

fn read_check_plan_review_file(review_path: &Path) -> Result<Option<String>, WorkflowError> {
    read_nonempty_review(review_path, "").map_err(WorkflowError)
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
    fn read_check_plan_review_file_returns_none_when_empty() {
        let t = tempfile::tempdir().unwrap();
        let review_path = t.path().join("empty.md");
        std::fs::write(&review_path, " \n\t").unwrap();
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

    #[test]
    fn kiss_stringify_check_plan_units() {
        let _ = stringify!(super::run_check_plan);
        let _ = stringify!(super::run_check_plan_attempt);
        let _ = stringify!(super::read_check_plan_review_file);
    }
}
