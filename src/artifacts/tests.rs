use super::*;

#[test]
fn log_path_sanitizes_slashes_and_backslashes() {
    let r = RunArtifacts {
        run_dir: PathBuf::from("/tmp/run"),
        plan_path: PathBuf::from("/tmp/run/plan.md"),
        work_dir: PathBuf::from("/work"),
    };
    assert_eq!(
        r.log_path("a/b").file_name(),
        Some(std::ffi::OsStr::new("a_b.log"))
    );
    assert_eq!(
        r.log_path("a\\b").file_name(),
        Some(std::ffi::OsStr::new("a_b.log"))
    );
}

#[test]
fn create_run_artifacts_relative_plan_uses_dot_work_dir() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    std::fs::write("plan.md", "restated request").unwrap();

    let art = create_run_artifacts(Path::new("plan.md"), None).unwrap();
    std::env::set_current_dir(old_cwd).unwrap();
    assert_eq!(art.work_dir, PathBuf::from("."));
}

#[test]
fn create_run_artifacts_from_text_uses_base_dir_as_work_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_run_artifacts_from_text("prompt", Some(tmp.path())).unwrap();
    assert_eq!(art.work_dir, tmp.path());
    assert_eq!(std::fs::read_to_string(&art.plan_path).unwrap(), "prompt");
}

#[test]
fn create_run_artifacts_opts_without_gc_skips_prune() {
    let tmp = tempfile::tempdir().unwrap();
    let logs = crate::malvin_logs_root(tmp.path());
    std::fs::create_dir_all(logs.join("20260101_000000_keepkeep")).unwrap();
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").unwrap();
    let art = create_run_artifacts_opts(
        &plan,
        Some(tmp.path()),
        crate::run_id::RunDirOptions::without_gc(),
    )
    .unwrap();
    assert!(logs.join("20260101_000000_keepkeep").exists());
    assert!(art.run_dir.starts_with(&logs));
}

#[test]
fn create_run_artifacts_from_text_opts_without_gc() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_run_artifacts_from_text_opts(
        "prompt",
        Some(tmp.path()),
        crate::run_id::RunDirOptions::without_gc(),
    )
    .unwrap();
    assert_eq!(art.work_dir, tmp.path());
}

#[test]
fn create_kpop_run_artifacts_opts_without_gc() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_kpop_run_artifacts_opts(
        "kpop body",
        Some(tmp.path()),
        crate::run_id::RunDirOptions::without_gc(),
    )
    .unwrap();
    assert!(art.plan_path.ends_with("request.md"));
}

#[test]
fn create_kpop_run_artifacts_writes_request_md() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_kpop_run_artifacts("kpop body", Some(tmp.path())).unwrap();
    assert!(art.plan_path.ends_with("request.md"));
    assert!(art.exp_log_path().is_file());
    assert_eq!(
        std::fs::read_to_string(&art.plan_path).unwrap(),
        "kpop body"
    );
}

#[test]
fn work_dir_for_path_uses_parent_or_dot() {
    assert_eq!(work_dir_for_path(Path::new("a/b.md")), PathBuf::from("a"));
    assert_eq!(work_dir_for_path(Path::new("plan.md")), PathBuf::from("."));
}

#[test]
fn resolve_user_md_request_literal_uses_dot_work_dir_and_trims() {
    let (text, wd) = resolve_user_md_request("  hello world  ").unwrap();
    assert_eq!(text, "hello world");
    assert_eq!(wd, PathBuf::from("."));
}

#[test]
fn resolve_user_md_request_reads_existing_md_file() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    std::fs::write("note.md", "line1\n").unwrap();
    let abs_file = tmp.path().join("note.md");
    let (text, wd) = resolve_user_md_request("note.md").unwrap();
    std::env::set_current_dir(old_cwd).unwrap();
    assert_eq!(text, "line1\n");
    assert_eq!(
        wd,
        work_dir_for_path(&abs_file),
        "absolute and cwd-relative resolution must agree on work_dir"
    );
}

#[test]
fn is_existing_md_file_path_rejects_invalid_and_directory() {
    assert!(is_existing_md_file_path("fix plan.md").is_none());
    assert!(is_existing_md_file_path("missing.md").is_none());
    assert!(is_existing_md_file_path("../missing.md").is_none());
    assert!(is_existing_md_file_path("./note.md").is_none());
    assert!(is_existing_md_file_path("plan.MD").is_none());
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let dir = tmp.path().join("notes.md");
    std::fs::create_dir_all(&dir).unwrap();
    assert!(is_existing_md_file_path("notes.md").is_none());
    std::env::set_current_dir(old_cwd).unwrap();
}

