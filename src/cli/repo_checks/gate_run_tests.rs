use std::path::Path;

use crate::repo_checks::command_support::set_fake_command_dir;
use super::*;

#[cfg(unix)]
fn install_zero_exit_gate_bins(bin_dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    for name in ["kiss", "cargo", "ruff"] {
        let path = bin_dir.join(name);
        std::fs::write(&path, "#!/bin/sh\nexit 0\n").expect("write fake bin");
        let mut perms = std::fs::metadata(&path).expect("bin meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).expect("chmod fake bin");
    }
}

#[test]
fn shell_binary_returns_nonempty_names() {
    let (sh, arg) = shell_binary();
    assert!(!sh.is_empty());
    assert!(!arg.is_empty());
}

#[test]
fn source_like_files_absent_in_empty_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(!source_like_files_present(tmp.path()));
}

#[test]
fn prepare_repo_workspace_succeeds_on_empty_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    prepare_repo_workspace(tmp.path(), RepoGateOutput::Tagged, None).expect("prepare");
}

#[test]
fn gate_run_private_helpers_succeed_on_empty_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    prepare_repo_workspace_with_details(work, RepoGateOutput::Tagged, None, true).expect("prepare");
    ensure_kiss_clamp_if_needed_with_details(work, RepoGateOutput::Tagged, None)
        .expect("kiss clamp skipped without sources");
    run_malvin_checks_with_details(work, RepoGateOutput::Tagged, None, &[])
        .expect("empty malvin_checks");
    run_shell_command_line_with_details(work, RepoGateOutput::Tagged, None, "")
        .expect("empty shell line");
}

#[cfg(unix)]
#[test]
fn gate_run_wires_private_runners_on_minimal_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = \"m\"\nversion = \"0.1.0\"\n",
    )
    .expect("Cargo.toml");
    let bin_dir = tempfile::tempdir().expect("bindir");
    install_zero_exit_gate_bins(bin_dir.path());
    let _guard = set_fake_command_dir(bin_dir.path());

    run_quality_gates_with_details(work, RepoGateOutput::Tagged, None).expect("quality gates");
    run_repo_workspace_gates_with_details(work, RepoGateOutput::Tagged, None)
        .expect("workspace gates");
    run_repo_workspace_gates_no_kiss_clamp_with_details(work, RepoGateOutput::Tagged, None)
        .expect("workspace gates without kiss clamp");
}
