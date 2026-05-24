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
mod smoke_tests {
    use super::run_concerns_and_check_abort_impl;
    use crate::acp::{AgentClient, AgentIoOptions};
    use crate::artifacts::{
        KissConfigBackup, KissignoreBackup, MalvinChecksBackup, SessionDotfileBackups,
        create_run_artifacts_from_text,
    };
    use crate::orchestrator::{Orchestrator, WorkflowConfig, workflow_context};
    use crate::prompts::PromptStore;

    #[tokio::test]
    async fn concerns_step_errors_when_coder_session_not_open() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_run_artifacts_from_text("rlh_smoke", Some(tmp.path())).expect("art");
        let store = PromptStore::default_store();
        let ctx = workflow_context(&artifacts, &store, "code").expect("ctx");
        let mut client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                sandbox: false,
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
                dry_run: false,
            },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: SessionDotfileBackups::from_parts(
                KissConfigBackup::Missing,
                MalvinChecksBackup::Missing,
                KissignoreBackup::Missing,
            ),
        };
        let err = run_concerns_and_check_abort_impl(&mut orch, 1, "review_attempt_1", &ctx)
            .await
            .expect_err("concerns without session");
        assert!(
            err.0.contains("begin_coder_session"),
            "unexpected: {}",
            err.0
        );
    }
}
