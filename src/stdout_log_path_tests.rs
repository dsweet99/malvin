#[test]
fn stdout_log_path_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("out.log");
    crate::stdout_log_path::set_stdout_log_path(Some(path.clone()));
    assert_eq!(crate::stdout_log_path::clone_stdout_log_path(), Some(path));
    crate::stdout_log_path::set_stdout_log_path(None);
    assert!(crate::stdout_log_path::clone_stdout_log_path().is_none());
}
