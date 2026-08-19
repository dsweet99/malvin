use super::discover::path_is_executable as is_executable;

#[cfg(unix)]
#[test]
fn path_is_executable() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("tmpdir");
    let path = dir.path().join("codex");
    std::fs::write(&path, "").expect("write");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod");
    assert!(is_executable(&path));
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(&path, permissions).expect("chmod");
    assert!(!is_executable(&path));
}
