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

    #[tokio::test]
    async fn run_check_plan_spawn_fails() {
        use crate::acp::{AgentClient, AgentIoOptions};
        use crate::artifacts::{
            KissConfigBackup, KissignoreBackup, MalvinChecksBackup, SessionDotfileBackups,
            create_run_artifacts_from_text,
        };
        use crate::orchestrator::{Orchestrator, WorkflowConfig, workflow_context};
        use crate::prompts::PromptStore;

        use super::run_check_plan;

        let tmp = tempfile::tempdir().expect("tempdir");
        let store = PromptStore::default_store();
        let artifacts = create_run_artifacts_from_text("cp", Some(tmp.path())).expect("art");
        let ctx = workflow_context(&artifacts, &store, "plan").expect("ctx");
        let mut client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        let mut orch = Orchestrator {
            client: &mut client,
            prompts: &store,
            artifacts: &artifacts,
            config: WorkflowConfig {
                max_loops: 1,
                run_learn: false,
                learn_min_elapsed_ms: 0,
                skip_check_plan: false,
            },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: SessionDotfileBackups::from_parts(
                KissConfigBackup::Missing,
                MalvinChecksBackup::Missing,
                KissignoreBackup::Missing,
            ),
        };
        let err = run_check_plan(&mut orch, &ctx)
            .await
            .expect_err("check plan");
        assert!(!err.0.is_empty());
    }

    #[tokio::test]
    async fn run_check_plan_attempt_errors_when_spawn_fails() {
        use crate::acp::{AgentClient, AgentIoOptions};
        use crate::artifacts::{
            KissConfigBackup, KissignoreBackup, MalvinChecksBackup, SessionDotfileBackups,
            create_run_artifacts_from_text,
        };
        use crate::orchestrator::{Orchestrator, WorkflowConfig, workflow_context};
        use crate::prompts::PromptStore;

        use super::run_check_plan_attempt;

        let tmp = tempfile::tempdir().expect("tempdir");
        let store = PromptStore::default_store();
        let artifacts =
            create_run_artifacts_from_text("cp-attempt", Some(tmp.path())).expect("art");
        let ctx = workflow_context(&artifacts, &store, "plan").expect("ctx");
        let mut client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        let mut orch = Orchestrator {
            client: &mut client,
            prompts: &store,
            artifacts: &artifacts,
            config: WorkflowConfig {
                max_loops: 1,
                run_learn: false,
                learn_min_elapsed_ms: 0,
                skip_check_plan: false,
            },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: SessionDotfileBackups::from_parts(
                KissConfigBackup::Missing,
                MalvinChecksBackup::Missing,
                KissignoreBackup::Missing,
            ),
        };
        let review_path = artifacts.artifact_review_md();
        let err = run_check_plan_attempt(&mut orch, &ctx, &review_path)
            .await
            .expect_err("attempt");
        assert!(!err.0.is_empty());
    }
}
