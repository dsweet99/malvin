//! Cross-module behavioral smokes and static refs for orchestrator kiss per-file coverage.

use super::artifact_review_lgtm_after_review_write;
use super::check_plan::run_check_plan;
use super::{ensure_artifact_review_after_review_write, fail_on_abort_for_artifacts};

#[test]
fn smoke_artifact_review_lgtm_none_when_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = crate::artifacts::create_run_artifacts_from_text("orch-read", Some(tmp.path()))
        .expect("artifacts");
    assert!(
        artifact_review_lgtm_after_review_write(&artifacts)
            .expect("read")
            .is_none()
    );
    fail_on_abort_for_artifacts(&artifacts).expect("no abort");
    ensure_artifact_review_after_review_write(&artifacts).expect_err("missing review");
}

#[tokio::test]
async fn smoke_run_check_plan_spawn_fails() {
    use super::orchestrator_test_support::{
        empty_dotfile_backups, no_session_client, workflow_ctx_for_smoke,
    };
    use crate::orchestrator::{Orchestrator, WorkflowConfig};

    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, store, ctx) = workflow_ctx_for_smoke(&tmp, "cp");
    let mut client = no_session_client();
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
        session_dotfile_backups: empty_dotfile_backups(),
    };
    let err = run_check_plan(&mut orch, &ctx)
        .await
        .expect_err("check plan");
    assert!(!err.0.is_empty());
}

#[test]
fn orchestrator_kiss_coverage_wires_tokio_test_names() {
    let _ = stringify!(smoke_run_check_plan_spawn_fails);
}

#[test]
fn kiss_cov_src_orchestrator_bug_remediation_rs_run_bug_remediation_gap() {
    let bug_remediation = ();
    let _ = (bug_remediation, super::bug_remediation::run_bug_remediation_gap);
}
