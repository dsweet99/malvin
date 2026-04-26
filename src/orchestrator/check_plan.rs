use crate::artifacts::restore_workspace_grounding;
use crate::review_sync::is_lgtm_str;
use crate::run_timing::TimingPhase;
use std::collections::HashMap;
use std::path::Path;

use super::Orchestrator;
use super::WorkflowError;
use super::clear_review_file;

impl Orchestrator<'_> {
    async fn run_check_plan_attempt(
        &mut self,
        context: &HashMap<String, String>,
        review_path: &Path,
    ) -> Result<Option<String>, WorkflowError> {
        clear_review_file(review_path)
            .map_err(|e| WorkflowError(format!("failed to clear review file: {e}")))?;
        (self.progress_callback)("CheckPlan");
        self.run_coder_prompt("check_plan.md", context, "check", TimingPhase::CheckPlan)
            .await?;
        restore_workspace_grounding(&self.artifacts.work_dir, &self.grounding_backup)
            .map_err(WorkflowError)?;
        Ok(std::fs::read_to_string(review_path).ok())
    }

    pub(super) async fn run_check_plan(
        &mut self,
        context: &HashMap<String, String>,
    ) -> Result<(), WorkflowError> {
        let review_path = self.artifacts.artifact_review_md();
        for attempt in 0..super::CHECK_PLAN_MAX_ATTEMPTS {
            if attempt > 0 {
                (self.progress_callback)(
                    "CheckPlan: agent did not write review file, retrying",
                );
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            let Some(contents) = self.run_check_plan_attempt(context, &review_path).await?
            else {
                continue;
            };
            if is_lgtm_str(&contents) {
                return self.finish_check_plan_after_lgtm();
            }
            (self.progress_callback)(&format!("Plan check failed:\n{contents}"));
            return Err(WorkflowError("check_plan did not pass".to_string()));
        }
        Err(WorkflowError(
            "check_plan: agent did not write review file after retries".to_string(),
        ))
    }
}
