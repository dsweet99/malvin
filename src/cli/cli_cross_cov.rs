#[test]
fn smoke_format_gate_failures_non_empty() {
    let ws = super::format_workspace_gate_failure("malvin tidy", "gate failed");
    assert!(ws.contains("gate failed"));
    assert!(ws.contains("malvin tidy"));
}

#[cfg(unix)]
#[test]
fn smoke_has_source_files_detects_symlink_to_rs_in_workspace() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let real = outside.path().join("real.rs");
    std::fs::write(&real, "fn main() {}").unwrap();
    symlink(&real, tmp.path().join("linked.rs")).unwrap();
    assert!(crate::source_detect::has_source_files(tmp.path()));
}
