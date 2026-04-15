use crate::orchestrator::{WorkflowError, clear_review_file, prefer_primary_errors_over_timing, prompt_md_stem, should_run_learn_check, workflow_context};
use crate::review_sync::{is_lgtm, sync_review_file};
use crate::artifacts::RunArtifacts;
use crate::prompts::PromptStore;

fn tmp_review_paths() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    (t, workspace, artifact)
}

#[test]
fn prompt_md_stem_strips_suffix_without_panicking_on_short_names() {
    assert_eq!(prompt_md_stem("implement.md"), "implement");
    assert_eq!(prompt_md_stem("x.md"), "x");
    assert_eq!(prompt_md_stem(""), "");
    assert_eq!(prompt_md_stem("ab"), "ab");
    assert_eq!(prompt_md_stem("readme.markdown"), "readme.markdown");
}

#[test]
fn legacy_slice_stem_diverges_from_prompt_md_stem() {
    fn legacy_stem(s: &str) -> &str {
        &s[..s.len().saturating_sub(3)]
    }
    assert_eq!(legacy_stem("review_1.md"), prompt_md_stem("review_1.md"));
    assert_eq!(legacy_stem("review_2.md"), prompt_md_stem("review_2.md"));
    assert_ne!(
        legacy_stem("readme.markdown"),
        prompt_md_stem("readme.markdown")
    );
    assert_ne!(legacy_stem("review_1.MD"), prompt_md_stem("review_1.MD"));
}

#[test]
fn is_lgtm_reads_file() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("r.md");
    std::fs::write(&p, "LGTM\n").unwrap();
    assert!(is_lgtm(&p));
}

#[test]
fn sync_review_file_clears_artifact_when_workspace_empty_so_stale_lgtm_is_removed() {
    let (_t, workspace, artifact) = tmp_review_paths();
    std::fs::write(&workspace, "").unwrap();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    sync_review_file(&workspace, &artifact).unwrap();
    assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "");
}

#[test]
fn sync_review_file_clears_artifact_when_workspace_whitespace_only() {
    let (_t, workspace, artifact) = tmp_review_paths();
    std::fs::write(&workspace, "  \n\t\n").unwrap();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    sync_review_file(&workspace, &artifact).unwrap();
    assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "");
}

#[test]
fn prefer_primary_errors_prefers_workflow_over_timing_when_both_fail() {
    let r = prefer_primary_errors_over_timing(
        Err(WorkflowError("workflow".into())),
        Ok(()),
        Err(WorkflowError("timing".into())),
    );
    assert_eq!(r.err().unwrap().0, "workflow");
}

#[test]
fn prefer_primary_errors_surfaces_timing_when_workflow_and_end_succeed() {
    let r = prefer_primary_errors_over_timing(Ok(()), Ok(()), Err(WorkflowError("timing".into())));
    assert_eq!(r.err().unwrap().0, "timing");
}

#[test]
fn sync_review_file_copies_nonempty_workspace_to_artifact() {
    let (_t, workspace, artifact) = tmp_review_paths();
    std::fs::write(&workspace, "LGTM\n").unwrap();
    std::fs::write(&artifact, "old").unwrap();
    sync_review_file(&workspace, &artifact).unwrap();
    assert_eq!(std::fs::read_to_string(&artifact).unwrap().trim(), "LGTM");
}

#[test]
fn workflow_context_review_path_must_point_to_workspace_not_artifact() {
    let t = tempfile::tempdir().unwrap();
    let run_dir = t.path().join("_malvin").join("run123");
    std::fs::create_dir_all(&run_dir).unwrap();
    let plan_path = run_dir.join("plan.md");
    std::fs::write(&plan_path, "test plan").unwrap();

    let artifacts = RunArtifacts {
        run_dir,
        plan_path,
        work_dir: t.path().to_path_buf(),
    };
    let prompts = PromptStore::default_store();
    let ctx = workflow_context(&artifacts, &prompts);

    let review_path = ctx.get("review_path").expect("review_path must be in context");

    // The review_path should point to workspace review.md (./review.md),
    // NOT the artifact review.md (./_malvin/run123/review.md).
    // This is critical because sync_review_file reads from workspace and writes to artifact.
    // If the prompt tells the agent to write to the artifact path, sync will clear it
    // because the workspace file doesn't exist.
    assert!(
        !review_path.contains("_malvin"),
        "review_path must point to workspace (./review.md), not artifact (./_malvin/.../review.md); \
         got: {review_path}"
    );
    assert_eq!(
        review_path, "./review.md",
        "review_path should be ./review.md (workspace path)"
    );
}

#[test]
fn should_run_learn_check_zero_threshold_always_runs() {
    assert!(should_run_learn_check(0, 0), "0 threshold, 0 elapsed => run");
    assert!(should_run_learn_check(0, 1), "0 threshold, any elapsed => run");
    assert!(should_run_learn_check(0, 300_000), "0 threshold, 5 min => run");
}

#[test]
fn should_run_learn_check_below_threshold_skips() {
    assert!(!should_run_learn_check(300_000, 0), "5 min threshold, 0 elapsed => skip");
    assert!(!should_run_learn_check(300_000, 299_999), "5 min threshold, just under => skip");
}

#[test]
fn should_run_learn_check_at_or_above_threshold_runs() {
    assert!(should_run_learn_check(300_000, 300_000), "5 min threshold, exactly 5 min => run");
    assert!(should_run_learn_check(300_000, 300_001), "5 min threshold, just over => run");
    assert!(should_run_learn_check(300_000, 600_000), "5 min threshold, 10 min => run");
}

#[test]
fn clear_review_file_removes_existing_lgtm_content() {
    let t = tempfile::tempdir().unwrap();
    let review_path = t.path().join("review.md");
    std::fs::write(&review_path, "LGTM\n").unwrap();
    assert!(is_lgtm(&review_path), "precondition: file contains LGTM");
    clear_review_file(&review_path);
    assert!(!review_path.exists(), "clear_review_file should remove file");
    assert!(!is_lgtm(&review_path), "is_lgtm returns false after clear");
}

#[test]
fn clear_review_file_succeeds_on_nonexistent_file() {
    let t = tempfile::tempdir().unwrap();
    let review_path = t.path().join("does_not_exist.md");
    clear_review_file(&review_path);
    assert!(!review_path.exists());
}

#[test]
fn check_plan_error_message_format() {
    let err = WorkflowError("check_plan did not pass".to_string());
    assert_eq!(err.0, "check_plan did not pass");
}
