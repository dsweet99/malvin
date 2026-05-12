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
fn resolve_user_request_literal_uses_dot_work_dir_and_trims() {
    let (text, wd) = resolve_user_request("  hello world  ").unwrap();
    assert_eq!(text, "hello world");
    assert_eq!(wd, PathBuf::from("."));
}

#[test]
fn resolve_user_request_at_file_reads_contents_and_parent_work_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let f = tmp.path().join("note.md");
    std::fs::write(&f, "line1\n").unwrap();
    let arg = format!("@{}", f.display());
    let (text, wd) = resolve_user_request(&arg).unwrap();
    assert_eq!(text, "line1\n");
    assert_eq!(wd, tmp.path());
}

#[test]
fn resolve_user_request_at_missing_file_errors() {
    let err = resolve_user_request("@/no/such/file/plan_zz.md").unwrap_err();
    assert!(err.contains("does not exist"), "unexpected err: {err}");
}

#[test]
fn resolve_user_request_at_empty_path_errors() {
    let err = resolve_user_request("@").unwrap_err();
    assert_eq!(err, "Empty path after `@`.");
}
