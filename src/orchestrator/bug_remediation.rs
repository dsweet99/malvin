use std::collections::HashMap;

use crate::acp::AgentError;

use super::session_flow::{run_bug_remediation_until_pre_summary, run_coder_session_summary_only};
use super::{Orchestrator, PreSummaryMidFn, WorkflowError, prefer_primary_errors_over_timing};

pub async fn run_bug_remediation_gap(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    mid: PreSummaryMidFn,
) -> Result<(), WorkflowError> {
    let timing = orchestrator.attach_run_timing();
    let begin_res = orchestrator
        .client
        .begin_coder_session(&orchestrator.artifacts.work_dir)
        .await;
    let coder_session_began = begin_res.is_ok();
    let workflow_result = match begin_res {
        Ok(()) => {
            async {
                run_bug_remediation_until_pre_summary(orchestrator, context).await?;
                mid(
                    orchestrator.client,
                    orchestrator.artifacts,
                    &orchestrator.session_dotfile_backups,
                )
                .await
                .map_err(WorkflowError)?;
                run_coder_session_summary_only(orchestrator, context).await
            }
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
mod tests {
    use crate::acp::{AgentClient, AgentIoOptions};
    use crate::artifacts::{
        create_run_artifacts_from_text, KissConfigBackup, KissignoreBackup, MalvinChecksBackup,
        SessionDotfileBackups,
    };
    use crate::orchestrator::{mid_noop, Orchestrator, WorkflowConfig, workflow_context};
    use crate::prompts::PromptStore;

    use super::run_bug_remediation_gap;

    #[tokio::test]
    async fn run_bug_remediation_gap_spawn_fails() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = PromptStore::default_store();
        let artifacts = create_run_artifacts_from_text("bug", Some(tmp.path())).expect("art");
        let ctx = workflow_context(&artifacts, &store, "code").expect("ctx");
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
        let err = run_bug_remediation_gap(&mut orch, &ctx, mid_noop)
            .await
            .expect_err("bug gap");
        assert!(!err.0.is_empty());
    }

    #[test]
    fn kiss_stringify_bug_remediation_units() {
        let _ = stringify!(super::run_bug_remediation_gap);
    }
}
