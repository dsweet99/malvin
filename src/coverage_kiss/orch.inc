// Orchestrator helper and review-kernel smoke tests.

use crate::artifacts::RunArtifacts;
use crate::orchestrator::{
    ReviewAttemptFinish, ReviewWriteInnerOutcome, WorkflowConfig, WorkflowError,
    artifact_review_lgtm_after_review_write, clear_review_attempt_artifacts,
    ensure_artifact_review_after_review_write, ensure_review_prep_after_reviewers_spawn,
    fail_on_abort_for_artifacts, format_exp_log_relative, format_prompt_path,
    is_missing_artifact_review_error, merge_string_run_and_restore, review_attempt_is_lgtm,
    should_run_learn_check, workflow_context, workflow_context_paths_only,
};

fn sample_run_artifacts(tmp: &tempfile::TempDir) -> RunArtifacts {
    let run_dir = tmp.path().join("_malvin").join("run1");
    std::fs::create_dir_all(&run_dir).expect("run dir");
    let plan_path = run_dir.join("plan.md");
    std::fs::write(&plan_path, "p").expect("plan");
    RunArtifacts {
        run_dir,
        plan_path,
        work_dir: tmp.path().to_path_buf(),
    }
}

#[test]
fn smoke_orchestrator_review_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = sample_run_artifacts(&tmp);
    let ctx = workflow_context_paths_only(&artifacts, "code");
    assert!(ctx.contains_key("plan_path"));

    let review_path = artifacts.artifact_review_md();
    std::fs::write(&review_path, "LGTM").expect("artifact review");
    assert!(review_attempt_is_lgtm(&artifacts).expect("read lgtm"));
    let lgtm_state = artifact_review_lgtm_after_review_write(&artifacts).expect("lgtm opt");
    assert_eq!(lgtm_state, Some(true));

    std::fs::write(&review_path, "").expect("clear review");
    let err = ensure_artifact_review_after_review_write(&artifacts).expect_err("missing art");
    assert!(is_missing_artifact_review_error(&err));

    std::fs::write(&review_path, "LGTM").expect("restore review");
    clear_review_attempt_artifacts(&artifacts).expect("clear attempt");
    let prep_err = ensure_review_prep_after_reviewers_spawn(&artifacts).expect_err("no prep");
    assert_eq!(
        prep_err.0,
        crate::orchestrator::REVIEW_PREP_MISSING_ARTIFACT_MSG
    );
}

#[test]
fn smoke_orchestrator_files_and_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = sample_run_artifacts(&tmp);
    fail_on_abort_for_artifacts(&artifacts).expect("no abort");
    assert!(crate::orchestrator::clear_review_file(&artifacts.workspace_review_md()).is_ok());
    assert!(crate::orchestrator::check_abort(&artifacts.artifact_result_md()).is_none());

    let exp = artifacts.exp_log_path();
    std::fs::create_dir_all(exp.parent().unwrap()).expect("kpop dir");
    std::fs::write(&exp, "x").expect("exp log");
    assert!(!format_exp_log_relative(&artifacts, &exp).is_empty());

    let sidecar = artifacts.run_dir.join("side.md");
    assert_eq!(
        format_prompt_path(&sidecar, tmp.path()),
        "./_malvin/run1/side.md"
    );
}

#[test]
fn smoke_orchestrator_types_and_merge() {
    merge_string_run_and_restore(Ok(()), Ok(())).expect("merge str");
    assert!(should_run_learn_check(0, 0));
    let _ = WorkflowConfig {
        max_loops: 1,
        run_learn: false,
        learn_min_elapsed_ms: crate::orchestrator::DEFAULT_LEARN_MIN_ELAPSED_MS,
        skip_check_plan: false,
    };
    let _ = WorkflowError("smoke".to_string());
    match ReviewAttemptFinish::Lgtm {
        ReviewAttemptFinish::Lgtm => {}
        ReviewAttemptFinish::NotLgtm | ReviewAttemptFinish::MissingArtifact => {
            panic!("unexpected variant");
        }
    }
    match ReviewWriteInnerOutcome::Lgtm {
        ReviewWriteInnerOutcome::Lgtm => {}
        ReviewWriteInnerOutcome::NotLgtm | ReviewWriteInnerOutcome::MissingArtifactExhausted => {
            panic!("unexpected variant");
        }
    }
    assert_eq!(crate::orchestrator::REVIEW_WRITE_INNER_RETRY_CAP, 2);
}

#[test]
fn smoke_workflow_context_full() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = sample_run_artifacts(&tmp);
    let prompts = PromptStore::default_store();
    let ctx = workflow_context(&artifacts, &prompts, "code").expect("workflow_context");

    assert!(
        ctx.contains_key("kpop") && !ctx.get("kpop").is_some_and(String::is_empty),
        "kpop slot rendered"
    );
    assert!(
        ctx.contains_key("quality_gates"),
        "quality_gates must be present"
    );
    assert_eq!(ctx.get("malvin_command").map(String::as_str), Some("code"));
    let review_path = ctx.get("review_path").expect("review_path");
    assert!(
        review_path.contains("_malvin") && review_path.ends_with("review.md"),
        "review_path should be artifact-relative; got {review_path}"
    );
}
