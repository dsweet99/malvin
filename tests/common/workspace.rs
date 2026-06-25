use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
pub fn chmod755(path: &Path) {
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

pub fn seed_malvin_checks(workspace: &Path, content: &str) {
    std::fs::create_dir_all(workspace.join(".malvin")).expect("mkdir .malvin");
    std::fs::write(workspace.join(".malvin/checks"), content).expect("write .malvin/checks");
}

pub fn seed_malvin_config(workspace: &Path, content: &str) {
    let path = malvin::malvin_config_path(workspace);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir ~/.malvin_home");
    }
    std::fs::write(path, content).expect("write ~/.malvin_home/config.toml");
}

/// Run `f` with `HOME` pointed at a fresh temp directory and restore afterward.
pub fn with_isolated_home<F>(f: F)
where
    F: FnOnce(&Path, &Path),
{
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    std::fs::create_dir_all(&home).expect("mkdir home");
    let work = root.path().join("work");
    std::fs::create_dir_all(&work).expect("mkdir work");
    let old_home = std::env::var_os("HOME");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("HOME", &home);
    }
    f(&work, &home);
    #[allow(unsafe_code)]
    unsafe {
        match old_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
    }
}

pub fn test_home_workspace() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    std::fs::write(workspace.join(".kissconfig"), "x").expect("kissconfig");
    (root, home, workspace)
}

#[cfg(unix)]
pub fn seed_git_kiss_cargo_gate_workspace(workspace: &Path) {
    std::fs::create_dir(workspace.join(".git")).expect("mkdir git marker");
    std::fs::write(
        workspace.join(".kissconfig"),
        "[gate]\ntest_coverage_threshold = 90\n",
    )
    .expect("write kissconfig");
    std::fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .expect("write cargo manifest");
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

/// Home-directory run logs bucket for `workspace` when `HOME` is set to `home`.
#[must_use]
pub fn malvin_run_logs_bucket(workspace: &Path, home: &Path) -> PathBuf {
    home.join(malvin::MALVIN_USER_HOME_DIR)
        .join("logs")
        .join(malvin::workspace_logs_hash(workspace))
}

#[cfg(unix)]
pub fn only_run_dir(workspace: &Path, home: &Path) -> PathBuf {
    let run_root = malvin_run_logs_bucket(workspace, home);
    let dirs: Vec<PathBuf> = std::fs::read_dir(&run_root)
        .expect("read home malvin logs bucket")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.is_dir())
        .collect();
    assert_eq!(dirs.len(), 1, "expected exactly one run dir, got {dirs:?}");
    dirs.into_iter().next().expect("run dir")
}

#[cfg(unix)]
pub fn write_failing_command(path: &Path, trace: &Path) {
    let name = path.file_name().unwrap().to_string_lossy();
    std::fs::write(
        path,
        format!(
            "#!/usr/bin/env sh\necho \"{name} $@\" >> \"{}\"\nexit 1\n",
            trace.display()
        ),
    )
    .expect("write failing command");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
pub fn write_failing_gate_tools(bin_dir: &Path, trace: &Path) {
    for name in ["kiss", "cargo", "ruff", "pytest"] {
        write_failing_command(&bin_dir.join(name), trace);
    }
}
