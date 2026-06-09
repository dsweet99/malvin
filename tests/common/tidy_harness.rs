use std::path::Path;
use std::process::Command;

use super::{
    INTEGRATION_TEST_MALVIN_ARGS, MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout,
    write_failing_gate_tools, write_fake_kiss,
};

pub struct TidySpawn<'a> {
    pub workspace: &'a Path,
    pub home: &'a Path,
    pub mock: &'a Path,
    pub path_var: &'a str,
    pub extra_args: &'a [&'a str],
}

pub fn spawn_tidy(t: &TidySpawn<'_>) -> std::process::Output {
    spawn_tidy_with_timeout(t, MALVIN_TEST_CMD_TIMEOUT)
}

pub fn spawn_tidy_with_timeout(
    t: &TidySpawn<'_>,
    timeout: std::time::Duration,
) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(t.workspace)
        .env("HOME", t.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", t.mock)
        .env("PATH", t.path_var);
    let mut args: Vec<&str> = vec!["tidy"];
    args.extend_from_slice(INTEGRATION_TEST_MALVIN_ARGS);
    args.extend_from_slice(t.extra_args);
    cmd.args(args);
    command_output_with_timeout(&mut cmd, timeout).expect("spawn malvin")
}

pub fn workspace_kiss_check_only(workspace: &Path) {
    super::seed_malvin_checks(workspace, "kiss check\n");
}

pub fn bin_path_with_failing_gates(root: &tempfile::TempDir, trace: &Path) -> String {
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_failing_gate_tools(&bin_dir, trace);
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

pub fn write_kiss_fail_until_n_passes(path: &Path, trace: &Path, fail_count: u32) {
    let trace = trace.display();
    let script = format!(
        "#!/usr/bin/env sh\n\
n=0\n\
if [ -f '{trace}' ]; then n=$(cat '{trace}'); fi\n\
n=$((n + 1))\n\
echo \"$n\" > '{trace}'\n\
if [ \"$n\" -le {fail_count} ]; then exit 1; fi\n\
exit 0\n"
    );
    std::fs::write(path, script).expect("write kiss");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    std::fs::set_permissions(path, perms).expect("chmod");
}

pub fn bin_path_with_kiss_fail_until_n_passes(
    root: &tempfile::TempDir,
    trace: &Path,
    fail_count: u32,
) -> String {
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_kiss_fail_until_n_passes(&bin_dir.join("kiss"), trace, fail_count);
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

pub fn bin_path_with_fake_kiss(root: &tempfile::TempDir) -> String {
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_fake_kiss(&bin_dir.join("kiss"));
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}
