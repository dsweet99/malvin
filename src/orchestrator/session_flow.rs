use std::collections::HashMap;

use crate::run_timing::TimingPhase;

use super::{Orchestrator, WorkflowError};

pub(super) async fn run_coder_session_summary_only(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    (orchestrator.progress_callback)("Summary");
    let summary_body = orchestrator
        .prompts
        .render("summary.md", context)
        .map_err(|e| WorkflowError(e.0))?;
    orchestrator
        .run_coder_prompt_body(summary_body, "summary.md", "summary", TimingPhase::Summary)
        .await?;
    orchestrator.fail_on_abort_result()?;
    Ok(())
}

#[cfg(test)]
mod session_flow_smoke_tests {
    use super::run_coder_session_summary_only;
    use crate::acp::AgentClient;
    use crate::artifacts::RunArtifacts;
    use crate::orchestrator::orchestrator_test_support::{
        empty_dotfile_backups, no_session_client, workflow_ctx_for_smoke,
    };
    use crate::orchestrator::{Orchestrator, WorkflowConfig};
    use crate::prompts::PromptStore;

    fn mk_orchestrator<'a>(
        client: &'a mut AgentClient,
        store: &'a PromptStore,
        artifacts: &'a RunArtifacts,
    ) -> Orchestrator<'a> {
        Orchestrator {
            client,
            prompts: store,
            artifacts,
            config: WorkflowConfig { max_loops: 1 },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: empty_dotfile_backups(),
        }
    }

    #[tokio::test]
    async fn summary_only_errors_when_coder_session_not_open() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts, store, ctx) = workflow_ctx_for_smoke(&tmp, "sf_smoke");
        let mut client = no_session_client();
        let mut orch = mk_orchestrator(&mut client, &store, &artifacts);
        let err = run_coder_session_summary_only(&mut orch, &ctx)
            .await
            .expect_err("expected prompt without session");
        assert!(
            err.0.contains("begin_coder_session"),
            "unexpected err: {}",
            err.0
        );
    }
}
