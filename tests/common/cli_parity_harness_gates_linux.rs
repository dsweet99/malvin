#[cfg(all(unix, target_os = "linux"))]
use std::os::unix::fs::PermissionsExt;
#[cfg(all(unix, target_os = "linux"))]
use std::path::Path;
#[cfg(all(unix, target_os = "linux"))]
use std::process::Command;

#[cfg(all(unix, target_os = "linux"))]
pub fn assert_workspace_gate_trace(trace_log: &str) {
    for needle in [
        "kiss clamp",
        "kiss check",
        "cargo clippy",
        "cargo test",
        "ruff check",
        "pytest -sv tests",
    ] {
        assert!(
            trace_log.contains(needle),
            "pre-run quality gates should include {needle}: {trace_log:?}"
        );
    }
}

#[cfg(all(unix, target_os = "linux"))]
fn seed_ground_gates_rust_stub(workspace: &Path) {
    std::fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .expect("write cargo");
    std::fs::create_dir_all(workspace.join("src")).expect("mkdir src");
    std::fs::write(workspace.join("src/main.rs"), "fn main() {}\n").expect("write src");
}

#[cfg(all(unix, target_os = "linux"))]
fn seed_ground_gates_python_stub(workspace: &Path) {
    std::fs::write(workspace.join("script.py"), "print('ok')\n").expect("write python");
    std::fs::create_dir_all(workspace.join("tests")).expect("mkdir tests");
    std::fs::write(
        workspace.join("tests/test_m.py"),
        "def test_x():\n    pass\n",
    )
    .expect("write test");
}

#[cfg(all(unix, target_os = "linux"))]
pub fn seed_ground_gates_workspace(workspace: &Path) {
    std::fs::create_dir_all(workspace.join(".git")).expect("mkdir .git");
    seed_ground_gates_rust_stub(workspace);
    seed_ground_gates_python_stub(workspace);
    std::fs::remove_file(workspace.join("grounding.md")).expect("remove grounding");
}

#[cfg(all(unix, target_os = "linux"))]
pub fn write_fake_command_trace(path: &Path, trace: &Path) {
    let name = path.file_name().unwrap().to_string_lossy();
    std::fs::write(
        path,
        format!(
            "#!/usr/bin/env sh\necho \"{name} $@\" >> \"{}\"\nexit 0\n",
            trace.display()
        ),
    )
    .expect("write fake command");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(all(unix, target_os = "linux"))]
fn write_fake_gate_command_traces(bin_dir: &Path, trace: &Path) {
    for name in ["kiss", "cargo", "ruff", "pytest"] {
        write_fake_command_trace(&bin_dir.join(name), trace);
    }
}

#[cfg(all(unix, target_os = "linux"))]
fn spawn_malvin_ground(
    workspace: &Path,
    home: &Path,
    mock: &Path,
    path: &str,
) -> std::process::Output {
    super::command_output_with_timeout(
        Command::new(env!("CARGO_BIN_EXE_malvin"))
            .current_dir(workspace)
            .env("HOME", home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", mock)
            .env("PATH", path)
            .args(["ground"]),
        super::MALVIN_TEST_CMD_TIMEOUT,
    )
    .expect("spawn malvin ground")
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_ground_with_fake_gate_trace(
    mock_js: &str,
) -> (
    std::process::Output,
    tempfile::TempDir,
    std::path::PathBuf,
) {
    let (root, home, workspace) = super::test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join("mock-agent-acp-ground-gates");
    super::write_mock_executable(&mock, mock_js);
    let trace = root.path().join("quality-trace.log");
    write_fake_gate_command_traces(&bin_dir, &trace);
    seed_ground_gates_workspace(&workspace);
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = spawn_malvin_ground(&workspace, &home, &mock, &path);
    (out, root, trace)
}
