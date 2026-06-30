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
    if std::process::Command::new("git")
        .args(["init"])
        .current_dir(workspace)
        .status()
        .is_ok_and(|s| s.success())
        || workspace.join(".git").exists()
    {
        // git-root layout
    }
    let checks_path = malvin::malvin_checks_path(workspace);
    if let Some(parent) = checks_path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir checks parent");
    }
    std::fs::write(checks_path, content).expect("write checks");
}

/// Requires isolated `HOME`; see plan.md.
pub fn seed_malvin_config(workspace: &Path, content: &str) {
    assert!(
        std::env::var(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION).as_deref() == Ok("1"),
        "seed_malvin_config requires with_isolated_home or activate_test_home (see plan.md)"
    );
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
    let old_home_config_mutation = std::env::var_os(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION);
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("HOME", &home);
        std::env::set_var(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION, "1");
    }
    seed_fast_integration_malvin_config(&home);
    f(&work, &home);
    restore_env_var("HOME", old_home);
    restore_env_var(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION, old_home_config_mutation);
}

/// Point `$HOME` at an isolated temp home and allow home-config restore/repair to mutate it.
pub fn activate_test_home(home: &Path) {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("HOME", home);
        std::env::set_var(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION, "1");
    }
    seed_fast_integration_malvin_config(home);
}

/// Isolated `$HOME` config for integration subprocess tests: disable MPC planner sessions
/// (default `mpc = true` doubles ACP mock invocations per gate-loop iteration).
pub fn seed_fast_integration_malvin_config(home: &Path) {
    seed_malvin_config(home, "mpc = false\n");
}

fn restore_env_var(key: &str, value: Option<std::ffi::OsString>) {
    #[allow(unsafe_code)]
    unsafe {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
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
    let old_home = std::env::var_os("HOME");
    let old_mutation = std::env::var_os(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION);
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("HOME", &home);
        std::env::set_var(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION, "1");
    }
    seed_fast_integration_malvin_config(&home);
    restore_env_var("HOME", old_home);
    restore_env_var(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION, old_mutation);
    (root, home, workspace)
}

#[cfg(unix)]
pub fn seed_git_kiss_cargo_gate_workspace(workspace: &Path) {
    seed_git_gate_workspace_cached(workspace);
}

#[cfg(unix)]
fn gate_workspace_fixture_root() -> &'static Path {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gate_workspace")
    })
    .as_path()
}

#[cfg(unix)]
pub fn seed_git_gate_workspace_cached(workspace: &Path) {
    let fixture = gate_workspace_fixture_root();
    copy_dir_recursive(fixture, workspace).expect("copy gate workspace fixture");
}

#[cfg(unix)]
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let from = entry.path();
            let to = dst.join(entry.file_name());
            if file_type.is_dir() {
                copy_dir_recursive(&from, &to)?;
            } else {
                std::fs::copy(&from, &to)?;
            }
        }
    }
    Ok(())
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
    chmod755(path);
}

#[cfg(unix)]
fn mock_cache_key(js: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    js.hash(&mut hasher);
    hasher.finish()
}

#[cfg(unix)]
fn mock_cache_root() -> &'static tempfile::TempDir {
    static ROOT: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| tempfile::tempdir().expect("mock cache tempdir"))
}

#[cfg(unix)]
pub fn cached_mock_executable(js: &str) -> std::path::PathBuf {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static CACHE: OnceLock<Mutex<HashMap<u64, std::path::PathBuf>>> = OnceLock::new();

    let key = mock_cache_key(js);
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().expect("mock cache lock");
    if let Some(path) = guard.get(&key) {
        return path.clone();
    }

    let path = mock_cache_root().path().join(format!("mock-{key:x}"));
    if !path.is_file() {
        write_mock_executable(&path, js);
    }
    guard.insert(key, path.clone());
    path
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
