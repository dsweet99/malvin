#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::path::{Path, PathBuf};

pub fn test_home_workspace() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    std::fs::write(workspace.join("grounding.md"), "x").expect("grounding");
    (root, home, workspace)
}

#[cfg(unix)]
pub fn write_fake_kiss(path: &std::path::Path) {
    std::fs::write(path, "#!/usr/bin/env sh\nexit 0\n").expect("write kiss");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
pub fn write_mock_executable(path: &std::path::Path, js: &str) {
    let script = format!("#!/usr/bin/env node\n{js}");
    std::fs::write(path, script).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
pub fn only_run_dir(workspace: &Path) -> PathBuf {
    let run_root = workspace.join("_malvin");
    let dirs: Vec<PathBuf> = std::fs::read_dir(&run_root)
        .expect("read _malvin")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.is_dir())
        .collect();
    assert_eq!(dirs.len(), 1, "expected exactly one run dir, got {dirs:?}");
    dirs.into_iter().next().expect("run dir")
}
