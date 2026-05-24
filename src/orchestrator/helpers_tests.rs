use super::*;
use std::collections::HashMap;

#[test]
fn insert_formatted_adds_formatted_path() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("file.md");
    std::fs::write(&path, "").unwrap();
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "key", &path, tmp.path());
    assert!(ctx.get("key").unwrap().contains("file.md"));
}

#[test]
fn format_prompt_path_relative_when_target_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join(".malvin/logs").join("run123");
    std::fs::create_dir_all(&run_dir).unwrap();
    let review = run_dir.join("review.md");
    assert_eq!(
        format_prompt_path(&review, tmp.path()),
        "./.malvin/logs/run123/review.md"
    );
}

fn format_prompt_path_from_alternate_cwd(relative: &Path, base: &Path, cwd: &Path) -> String {
    let original = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(cwd).expect("chdir");
    let formatted = format_prompt_path(relative, base);
    std::env::set_current_dir(original).expect("restore cwd");
    formatted
}

#[test]
fn format_prompt_path_relative_existing_file_resolves_against_base_not_cwd() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("workspace");
    let other = tmp.path().join("other");
    std::fs::create_dir_all(&work).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(work.join("review.md"), "").unwrap();
    std::fs::write(other.join("review.md"), "").unwrap();
    let formatted = format_prompt_path_from_alternate_cwd(Path::new("review.md"), &work, &other);
    assert_eq!(
        formatted, "./review.md",
        "relative paths must resolve against base_dir, not process cwd"
    );
}

#[test]
fn check_abort_returns_message_after_prefix_not_entire_file() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("result.md");
    std::fs::write(&p, "context line\nABORT: stop here\nmore\n").unwrap();
    assert_eq!(check_abort(&p).as_deref(), Some("stop here"));
}

#[test]
fn check_abort_returns_none_when_no_abort_line() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("result.md");
    std::fs::write(&p, "ok\n").unwrap();
    assert!(check_abort(&p).is_none());
}

#[test]
fn check_abort_strips_utf8_bom_before_matching_abort_line() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("result.md");
    let mut bytes: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"ABORT: bom case\n");
    std::fs::write(&p, bytes).unwrap();
    assert_eq!(check_abort(&p).as_deref(), Some("bom case"));
}

#[test]
fn insert_artifact_paths_populates_context() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join(".malvin/logs").join("run");
    std::fs::create_dir_all(&run_dir).unwrap();
    let plan_path = run_dir.join("plan.md");
    std::fs::write(&plan_path, "plan").unwrap();
    let artifacts = crate::artifacts::RunArtifacts {
        run_dir,
        plan_path,
        work_dir: tmp.path().to_path_buf(),
    };
    let ctx = workflow_context_paths_only(&artifacts, "code");
    assert!(ctx.contains_key("plan_path"));
    assert!(ctx.contains_key("kpop_log_dir"));
    assert!(ctx.contains_key("review_path"));
    assert!(ctx.contains_key("result_path"));
    assert!(ctx.contains_key("quality_gates_log"));
    assert_eq!(ctx.get("malvin_command").map(String::as_str), Some("code"));
}

#[test]
fn summary_render_uses_malvin_output_path_run_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let artifacts =
        crate::artifacts::create_run_artifacts_from_text("plan", Some(tmp.path())).unwrap();
    let store = crate::prompts::PromptStore::default_store();
    let ctx = workflow_context(&artifacts, &store, "code").unwrap();
    let rendered = store.render("summary.md", &ctx).unwrap();
    assert!(
        rendered.contains("Review the files in ./.malvin/logs/"),
        "summary must point at the run directory, got {rendered:?}"
    );
    assert!(
        !rendered.contains("Review the files in .  Summarize"),
        "summary must not collapse malvin_output_path to dot, got {rendered:?}"
    );
    assert!(
        !rendered.contains("{{ malvin_output_path }}"),
        "summary must render malvin_output_path, got {rendered:?}"
    );
}

#[test]
fn format_prompt_path_uses_display_when_not_under_base() {
    let tmp = tempfile::tempdir().unwrap();
    let outside = std::env::temp_dir().join(format!("malvin_outside_{}", std::process::id()));
    std::fs::create_dir_all(&outside).unwrap();
    let review = outside.join("review.md");
    std::fs::write(&review, "").unwrap();
    let formatted = format_prompt_path(&review, tmp.path());
    assert!(
        !formatted.starts_with("./"),
        "paths outside base_dir must not be forced to ./ relative form: {formatted}"
    );
    assert!(
        formatted.contains("review.md"),
        "fallback display path must still name the file: {formatted}"
    );
    let _ = std::fs::remove_dir_all(&outside);
}
