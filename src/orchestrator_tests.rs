use crate::orchestrator::{DEFAULT_LEARN_MIN_ELAPSED_MS, should_run_learn_check};
use crate::artifacts::RunArtifacts;
use crate::orchestrator::{
    WorkflowError, clear_review_file, prefer_primary_errors_over_timing, prompt_md_stem,
    workflow_context,
};
use crate::prompts::PromptStore;
use crate::review_sync::{is_lgtm, sync_review_file};

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
    assert_eq!(
        legacy_stem("reviewers_spawn.md"),
        prompt_md_stem("reviewers_spawn.md")
    );
    assert_eq!(
        legacy_stem("review_write.md"),
        prompt_md_stem("review_write.md")
    );
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
fn prefer_primary_errors_chains_timing_when_workflow_fails() {
    let r = prefer_primary_errors_over_timing(
        Err(WorkflowError("workflow".into())),
        Ok(()),
        Err(WorkflowError("timing".into())),
    );
    assert_eq!(r.err().unwrap().0, "workflow; timing: timing");
}

#[test]
fn prefer_primary_errors_omits_timing_suffix_when_timing_succeeds() {
    let r =
        prefer_primary_errors_over_timing(Err(WorkflowError("workflow".into())), Ok(()), Ok(()));
    assert_eq!(r.err().unwrap().0, "workflow");
}

#[test]
fn prefer_primary_errors_surfaces_timing_when_workflow_and_end_succeed() {
    let r = prefer_primary_errors_over_timing(Ok(()), Ok(()), Err(WorkflowError("timing".into())));
    assert_eq!(r.err().unwrap().0, "timing");
}

#[test]
fn prefer_primary_errors_chains_timing_when_end_fails() {
    let r = prefer_primary_errors_over_timing(
        Ok(()),
        Err(WorkflowError("end".into())),
        Err(WorkflowError("timing".into())),
    );
    assert_eq!(r.err().unwrap().0, "end; timing: timing");
}

#[test]
fn prefer_primary_errors_chains_timing_when_workflow_and_end_fail() {
    let r = prefer_primary_errors_over_timing(
        Err(WorkflowError("workflow".into())),
        Err(WorkflowError("end".into())),
        Err(WorkflowError("timing".into())),
    );
    assert_eq!(r.err().unwrap().0, "workflow; end: end; timing: timing");
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
fn workflow_context_review_path_points_to_artifact() {
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
    let ctx = workflow_context(&artifacts, &prompts, "code").expect("workflow_context");

    let review_path = ctx
        .get("review_path")
        .expect("review_path must be in context");

    assert!(
        review_path.contains("_malvin"),
        "review_path must point to artifact (./_malvin/.../review.md); got: {review_path}"
    );
    assert_eq!(
        review_path, "./_malvin/run123/review.md",
        "review_path should be the artifact path"
    );
    assert!(
        ctx.contains_key("quality_gates"),
        "quality_gates must be in context"
    );
    assert_eq!(
        ctx.get("quality_gates_log").map(String::as_str),
        Some("./_malvin/run123/quality_gates.log"),
        "quality_gates_log should point to the run artifact log"
    );
}

#[test]
fn workflow_context_includes_malvin_command() {
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
    let ctx = workflow_context(&artifacts, &prompts, "tidy").expect("workflow_context");
    assert_eq!(ctx.get("malvin_command").map(String::as_str), Some("tidy"));
}

#[test]
fn clear_review_file_removes_existing_lgtm_content() {
    let t = tempfile::tempdir().unwrap();
    let review_path = t.path().join("review.md");
    std::fs::write(&review_path, "LGTM\n").unwrap();
    assert!(is_lgtm(&review_path), "precondition: file contains LGTM");
    clear_review_file(&review_path).unwrap();
    assert!(
        !review_path.exists(),
        "clear_review_file should remove file"
    );
    assert!(!is_lgtm(&review_path), "is_lgtm returns false after clear");
}

#[test]
fn clear_review_file_succeeds_on_nonexistent_file() {
    let t = tempfile::tempdir().unwrap();
    let review_path = t.path().join("does_not_exist.md");
    clear_review_file(&review_path).unwrap();
    assert!(!review_path.exists());
}

#[test]
fn clear_review_file_returns_error_on_permission_denied() {
    use std::os::unix::fs::PermissionsExt;
    let t = tempfile::tempdir().unwrap();
    let protected_dir = t.path().join("protected");
    std::fs::create_dir(&protected_dir).unwrap();
    let review_path = protected_dir.join("review.md");
    std::fs::write(&review_path, "LGTM\n").unwrap();
    std::fs::set_permissions(&protected_dir, std::fs::Permissions::from_mode(0o000)).unwrap();
    let result = clear_review_file(&review_path);
    std::fs::set_permissions(&protected_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        result.is_err(),
        "clear_review_file should return error on permission denied"
    );
}

const FIVE_MIN_MS: u64 = DEFAULT_LEARN_MIN_ELAPSED_MS;

#[test]
fn should_run_learn_check_zero_threshold_always_runs() {
    assert!(should_run_learn_check(0, 0));
    assert!(should_run_learn_check(0, 1));
    assert!(should_run_learn_check(0, FIVE_MIN_MS));
}

#[test]
fn should_run_learn_check_below_threshold_skips() {
    assert!(!should_run_learn_check(FIVE_MIN_MS, 0));
    assert!(!should_run_learn_check(FIVE_MIN_MS, 299_999));
}

#[test]
fn should_run_learn_check_at_or_above_threshold_runs() {
    assert!(should_run_learn_check(FIVE_MIN_MS, FIVE_MIN_MS));
    assert!(should_run_learn_check(FIVE_MIN_MS, FIVE_MIN_MS + 1));
    assert!(should_run_learn_check(FIVE_MIN_MS, FIVE_MIN_MS * 2));
}
