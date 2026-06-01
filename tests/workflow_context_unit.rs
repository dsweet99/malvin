use std::path::Path;

use malvin::artifacts::create_run_artifacts;
use malvin::prompts::PromptStore;
use malvin::workflow_context::{
    format_prompt_path, workflow_context, workflow_context_paths_only,
};

#[test]
fn workflow_context_paths_include_plan() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = workflow_context_paths_only(&artifacts, "code");
    assert!(ctx.contains_key("plan_path"));
}

#[test]
fn format_prompt_path_strips_base() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let s = format_prompt_path(&plan, tmp.path());
    assert!(s.starts_with("./"));
}

#[test]
fn workflow_context_paths_include_review_and_gates_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = workflow_context_paths_only(&artifacts, "tidy");
    assert!(ctx.contains_key("review_path"));
    assert!(ctx.contains_key("quality_gates_log"));
    assert_eq!(
        ctx.get("quality_gates_path").map(String::as_str),
        ctx.get("quality_gates_log").map(String::as_str),
    );
    assert!(ctx.contains_key("kpop_log_dir"));
}

#[test]
fn workflow_context_paths_use_relative_prompt_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = workflow_context_paths_only(&artifacts, "tidy");
    let advice = ctx.get("advice_path").expect("advice_path");
    assert!(
        advice.starts_with("./"),
        "advice_path should be relative to work_dir, got {advice:?}"
    );
    let plan = ctx.get("plan_path").expect("plan_path");
    assert!(
        plan.starts_with("./") || plan.starts_with('/'),
        "plan_path should be workdir-relative or absolute (home logs), got {plan:?}"
    );
    for key in [
        "review_path",
        "review_prep_path",
        "result_path",
        "malvin_output_path",
        "quality_gates_log",
        "kpop_log_dir",
        "exp_log",
    ] {
        let path = ctx.get(key).unwrap_or_else(|| panic!("missing {key}"));
        assert!(
            path.starts_with("./") || path.starts_with('/'),
            "{key} should be workdir-relative or absolute (home logs), got {path:?}"
        );
        assert!(
            path.contains(".malvin"),
            "{key} should reference malvin paths, got {path:?}"
        );
    }
    assert_eq!(ctx.get("malvin_command").map(String::as_str), Some("tidy"));
}

#[test]
fn workflow_context_render_includes_kpop_and_quality_gates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").expect("write");
    let artifacts = create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let ctx = workflow_context(&artifacts, &store, "tidy").expect("context");
    assert!(ctx.contains_key("kpop"));
    assert!(ctx.contains_key("quality_gates"));
}

#[test]
fn format_prompt_path_resolves_relative_under_work_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sub = tmp.path().join("nested");
    std::fs::create_dir_all(&sub).expect("mkdir");
    let plan = sub.join("plan.md");
    std::fs::write(&plan, "x").expect("write");
    let formatted = format_prompt_path(Path::new("nested/plan.md"), tmp.path());
    assert!(formatted.contains("plan.md"));
    assert!(formatted.starts_with("./"));
}

#[test]
fn format_prompt_path_handles_non_canonical_file_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("pending").join("plan.md");
    std::fs::create_dir_all(plan.parent().expect("parent")).expect("mkdir");
    let formatted = format_prompt_path(&plan, tmp.path());
    assert!(formatted.contains("plan.md"));
}

#[test]
fn format_prompt_path_falls_back_when_outside_work_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let outside = std::env::temp_dir().join(format!("malvin-outside-{}", std::process::id()));
    std::fs::write(&outside, "x").expect("write outside");
    let formatted = format_prompt_path(&outside, tmp.path());
    assert!(!formatted.starts_with("./"));
    let _ = std::fs::remove_file(&outside);
}
