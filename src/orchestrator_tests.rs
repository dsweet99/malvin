use crate::artifacts::RunArtifacts;
use crate::orchestrator::{
    WorkflowError, clear_review_file, prefer_primary_errors_over_timing, prompt_md_stem,
    workflow_context,
};
use crate::prompts::PromptStore;
use crate::review_sync::{is_lgtm, sync_review_file};

fn tmp_review_artifact() -> (tempfile::TempDir, std::path::PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    (t, artifact)
}

#[test]
fn prompt_md_stem_strips_suffix_without_panicking_on_short_names() {
    assert_eq!(prompt_md_stem("bug_fix.md"), "bug_fix");
    assert_eq!(prompt_md_stem("x.md"), "x");
    assert_eq!(prompt_md_stem(""), "");
    assert_eq!(prompt_md_stem("ab"), "ab");
    assert_eq!(prompt_md_stem("readme.markdown"), "readme.markdown");
}

#[test]
fn is_lgtm_reads_file() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("r.md");
    std::fs::write(&p, "LGTM\n").unwrap();
    assert!(is_lgtm(&p));
}

#[test]
fn sync_review_file_returns_artifact_when_present() {
    let (_t, artifact) = tmp_review_artifact();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    let out = sync_review_file(&artifact).unwrap();
    assert_eq!(out.as_deref(), Some("LGTM\n"));
    assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "LGTM\n");
}

#[test]
fn sync_review_file_returns_none_when_artifact_whitespace_only() {
    let (_t, artifact) = tmp_review_artifact();
    std::fs::write(&artifact, "  \n\t\n").unwrap();
    let out = sync_review_file(&artifact).unwrap();
    assert_eq!(out, None);
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
fn sync_review_file_returns_nonempty_artifact_text() {
    let (_t, artifact) = tmp_review_artifact();
    std::fs::write(&artifact, "old").unwrap();
    let out = sync_review_file(&artifact).unwrap();
    assert_eq!(out.as_deref(), Some("old"));
    assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "old");
}

#[test]
fn workflow_context_review_path_points_to_artifact() {
    let t = tempfile::tempdir().unwrap();
    let run_dir = t.path().join(".malvin/logs").join("run123");
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
        review_path.contains(".malvin/logs"),
        "review_path must point to artifact (./.malvin/logs/.../review.md); got: {review_path}"
    );
    assert_eq!(
        review_path, "./.malvin/logs/run123/review.md",
        "review_path should be the artifact path"
    );
    assert!(
        ctx.contains_key("quality_gates"),
        "quality_gates must be in context"
    );
    assert_eq!(
        ctx.get("quality_gates_log").map(String::as_str),
        Some("./.malvin/logs/run123/quality_gates.log"),
        "quality_gates_log should point to the run artifact log"
    );
}

#[test]
fn workflow_context_includes_malvin_command() {
    let t = tempfile::tempdir().unwrap();
    let run_dir = t.path().join(".malvin/logs").join("run123");
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
