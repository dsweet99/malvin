use crate::acp::{AgentClient, AgentIoOptions};
use crate::artifacts::{
    create_run_artifacts_from_text, KissConfigBackup, KissignoreBackup, MalvinChecksBackup,
    SessionDotfileBackups,
};
use crate::orchestrator::{mid_noop, Orchestrator, WorkflowConfig, workflow_context};
use crate::prompts::PromptStore;

use crate::orchestrator::bug_remediation::run_bug_remediation_gap;

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
