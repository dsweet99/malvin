use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{
    format_prompt_path, insert_artifact_paths, insert_current_state, insert_formatted,
    resolve_nonexistent_path, resolve_path_against_base, workflow_context_paths_only,
};

#[test]
fn resolve_path_against_base_resolves_relative_plan_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path().canonicalize().expect("base");
    let resolved = resolve_path_against_base(Path::new("plan.md"), &base);
    assert!(resolved.ends_with("plan.md"));
}

#[test]
fn resolve_path_against_base_resolves_absolute_missing_file_under_base() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path().canonicalize().expect("base");
    let abs = tmp.path().join("src/foo.rs");
    let resolved = resolve_path_against_base(&abs, &base);
    assert!(
        resolved.starts_with(&base),
        "expected resolved path under base, got {}",
        resolved.display()
    );
    assert!(resolved.ends_with("src/foo.rs"));
}

#[test]
fn resolve_nonexistent_path_cases() {
    let _ = resolve_nonexistent_path;
    assert_eq!(resolve_nonexistent_path(Path::new("")), PathBuf::from(""));

    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path().canonicalize().expect("base");
    let missing = tmp.path().join("nested/missing.md");
    let resolved = resolve_nonexistent_path(&missing);
    assert!(resolved.starts_with(&base));
    assert!(resolved.ends_with("nested/missing.md"));

    let deep = tmp.path().join("a/b/c/d.md");
    let deep_resolved = resolve_nonexistent_path(&deep);
    assert!(deep_resolved.starts_with(&base));
    assert!(deep_resolved.ends_with("a/b/c/d.md"));
}

#[test]
fn format_prompt_path_fallback_uses_resolved_path_display() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let outside = std::env::temp_dir().join(format!("malvin_outside_{}", std::process::id()));
    std::fs::create_dir_all(&outside).expect("outside dir");
    let missing = outside.join("missing.md");
    let formatted = format_prompt_path(&missing, tmp.path());
    assert!(
        !formatted.starts_with("./"),
        "outside path must not be relativized: {formatted}"
    );
    assert!(
        formatted.contains("missing.md"),
        "fallback must name the file: {formatted}"
    );
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn insert_quality_gates_log_paths_sets_alias() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let mut ctx = HashMap::new();
    super::insert_quality_gates_log_paths(&mut ctx, &artifacts, tmp.path());
    assert_eq!(
        ctx.get("quality_gates_path").map(String::as_str),
        ctx.get("quality_gates_log").map(String::as_str),
    );
    assert!(ctx
        .get("quality_gates_log")
        .expect("log")
        .ends_with("quality_gates.log"));
}

#[test]
fn insert_artifact_paths_sets_logs_dir_to_home_bucket() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let mut ctx = HashMap::new();
    insert_artifact_paths(&mut ctx, &artifacts);
    let logs_dir = ctx.get("logs_dir").expect("logs_dir");
    let expected = format_prompt_path(&crate::malvin_logs_root(tmp.path()), tmp.path());
    assert_eq!(logs_dir, &expected);
    assert!(
        logs_dir.contains(".malvin_home/logs"),
        "logs_dir should point at home logs bucket, got {logs_dir:?}"
    );
}

#[test]
fn insert_artifact_paths_populates_expected_keys() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let mut ctx = HashMap::new();
    insert_artifact_paths(&mut ctx, &artifacts);
    assert!(ctx.contains_key("result_path"));
    assert!(ctx.contains_key("review_prep_path"));
    assert!(ctx.contains_key("advice_path"));
    assert!(ctx.contains_key("logs_dir"));
}

#[test]
fn workflow_context_paths_only_includes_current_state() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = workflow_context_paths_only(&artifacts, "code");
    assert!(ctx.contains_key("current_state"));
    assert!(ctx.get("current_state").expect("state").contains("User:"));
}

#[test]
fn insert_current_state_populates_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let mut ctx = HashMap::new();
    insert_current_state(&mut ctx, &artifacts, tmp.path());
    assert!(ctx
        .get("current_state")
        .expect("state")
        .contains("Sandbox memory:"));
}

#[test]
fn insert_formatted_stores_workflow_relative_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "plan_path", &plan, tmp.path());
    assert_eq!(ctx.get("plan_path").map(String::as_str), Some("./plan.md"));
}
