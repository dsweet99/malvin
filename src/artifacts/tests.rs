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
    assert_eq!(wd, work_dir_for_path(&f));
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

#[test]
fn resolve_user_at_path_rejects_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("plans");
    std::fs::create_dir_all(&dir).unwrap();
    let err = resolve_user_at_path(dir.to_str().unwrap()).unwrap_err();
    assert!(
        err.contains("directory") || err.contains("not a file"),
        "unexpected err: {err}"
    );
}

#[test]
fn resolve_user_at_path_joins_relative_to_cwd() {
    struct RestoreCwd(std::path::PathBuf);
    impl Drop for RestoreCwd {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }

    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let _restore = RestoreCwd(std::env::current_dir().unwrap());
    std::env::set_current_dir(tmp.path()).unwrap();
    std::fs::write("p.md", "x").unwrap();
    let path = resolve_user_at_path("p.md").expect("path");
    assert_eq!(
        path.canonicalize().expect("canonical"),
        tmp.path().join("p.md").canonicalize().expect("canonical")
    );
}

#[test]
fn create_kpop_run_artifacts_writes_request_md() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_kpop_run_artifacts("kpop body", Some(tmp.path())).unwrap();
    assert!(art.plan_path.ends_with("request.md"));
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
fn resolve_at_file_reads_existing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let f = tmp.path().join("doc.md");
    std::fs::write(&f, "hello").unwrap();
    let (text, wd) = resolve_at_file(&f.to_string_lossy()).unwrap();
    assert_eq!(text, "hello");
    assert_eq!(wd, work_dir_for_path(&f));
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
fn is_existing_md_file_path_rejects_whitespace_and_nonexistent() {
    assert!(is_existing_md_file_path("fix plan.md").is_none());
    assert!(is_existing_md_file_path("missing.md").is_none());
    assert!(is_existing_md_file_path("@plan.md").is_none());
    assert!(is_existing_md_file_path("../missing.md").is_none());
    assert!(is_existing_md_file_path("./note.md").is_none());
    assert!(is_existing_md_file_path("plan.MD").is_none());
}

#[test]
fn is_existing_md_file_path_rejects_directory() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let dir = tmp.path().join("notes.md");
    std::fs::create_dir_all(&dir).unwrap();
    assert!(is_existing_md_file_path("notes.md").is_none());
    std::env::set_current_dir(old_cwd).unwrap();
}
