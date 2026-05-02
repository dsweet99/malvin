use std::future::Future;
use std::pin::Pin;

use malvin::acp::AgentClient;
use malvin::artifacts::{GroundingBackup, RunArtifacts};

use super::repo_checks::{
    RepoGateFailure, RepoGateOutput, run_repo_workspace_gates, run_repo_workspace_gates_with_details,
};
use super::tidy_flow::run_tidy_prompt_after_post_run_gate_failure;

pub fn mid_pre_summary_repo_gates<'a>(
    client: &'a mut AgentClient,
    artifacts: &'a RunArtifacts,
    grounding_backup: &'a GroundingBackup,
) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(run_pre_summary_repo_gates_with_tidy_retry(
        client,
        artifacts,
        grounding_backup,
    ))
}

pub async fn run_pre_summary_repo_gates_with_tidy_retry(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    grounding_backup: &GroundingBackup,
) -> Result<(), String> {
    match run_repo_workspace_gates_with_details(
        &artifacts.work_dir,
        RepoGateOutput::Tagged,
        Some(&artifacts.run_dir),
    ) {
        Ok(()) => Ok(()),
        Err(RepoGateFailure::Command(failure)) => {
            run_tidy_prompt_after_post_run_gate_failure(
                client,
                artifacts,
                grounding_backup,
                &failure,
            )
            .await?;
            run_repo_workspace_gates(
                &artifacts.work_dir,
                RepoGateOutput::Tagged,
                Some(&artifacts.run_dir),
            )
            .map_err(|e| format!("post-run gates still failing after one tidy.md retry: {e}"))
        }
        Err(RepoGateFailure::Message(err)) => Err(err),
    }
}
