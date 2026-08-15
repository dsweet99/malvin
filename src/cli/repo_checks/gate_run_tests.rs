use super::*;
use crate::repo_checks::command_support::set_fake_command_dir;
use crate::test_stderr_capture::capture_stderr_output;
use crate::output::{format_who_tag_delim, ERROR_WHO, MALVIN_WHO, WARNING_WHO};

#[cfg(unix)]
fn install_zero_exit_gate_bins(bin_dir: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("lint");
    std::fs::write(&path, "#!/bin/sh\nexit 0\n").expect("write fake bin");
    let mut perms = std::fs::metadata(&path).expect("bin meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("chmod fake bin");
}

#[test]
fn shell_binary_returns_nonempty_names() {
    let (sh, arg) = shell_binary();
    assert!(!sh.is_empty());
    assert!(!arg.is_empty());
}

#[test]
fn gate_command_cwd_uses_work_dir_outside_git() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    assert_eq!(gate_command_cwd(work), work);
}

#[cfg(unix)]
#[test]
fn gate_commands_run_at_git_toplevel_from_nested_work_dir() {
    use std::fs;
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(root)
            .status()
            .expect("git init")
            .success()
    );
    let nested = root.join("nested");
    fs::create_dir_all(&nested).expect("mkdir nested");
    fs::write(root.join("ROOT_ONLY"), "x\n").expect("root marker");
    let checks = crate::malvin_checks_path(root);
    fs::create_dir_all(checks.parent().expect("checks parent")).expect("mkdir .malvin");
    // Relative path must resolve only when cwd is the git toplevel.
    fs::write(&checks, "test -f ROOT_ONLY\n").expect("write checks");

    run_repo_workspace_gates(&nested, RepoGateOutput::Tagged, None)
        .expect("gates from nested work_dir must run at git root");
    assert_eq!(
        gate_command_cwd(&nested)
            .canonicalize()
            .expect("canon nested cwd"),
        root.canonicalize().expect("canon root")
    );
}

#[test]
fn source_like_files_absent_in_empty_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(!crate::source_detect::has_source_files(tmp.path()));
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
    prepare_repo_workspace_with_details(work).expect("prepare");
    run_malvin_checks_with_details(work, RepoGateOutput::Tagged, None, &[])
        .expect("empty malvin_checks");
    run_shell_command_line_with_details(work, RepoGateOutput::Tagged, None, "")
        .expect("empty shell line");
}

#[cfg(unix)]
fn install_exit_one_gate_bin(bin_dir: &std::path::Path, name: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join(name);
    std::fs::write(&path, "#!/bin/sh\nexit 1\n").expect("write fake bin");
    let mut perms = std::fs::metadata(&path).expect("bin meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("chmod fake bin");
}

#[cfg(unix)]
#[test]
fn failing_gate_run_stderr_uses_malvin_not_error_or_warning() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    crate::seed_malvin_checks(work, "failgate\n");
    let bin_dir = tempfile::tempdir().expect("bindir");
    install_exit_one_gate_bin(bin_dir.path(), "failgate");
    let _guard = set_fake_command_dir(bin_dir.path());
    let malvin_tag = format_who_tag_delim(MALVIN_WHO);
    let error_tag = format_who_tag_delim(ERROR_WHO);
    let warning_tag = format_who_tag_delim(WARNING_WHO);
    let stderr = capture_stderr_output(|| {
        let _ = run_repo_workspace_gates(work, RepoGateOutput::Stderr, None);
    });
    assert!(
        stderr.contains(&malvin_tag) && stderr.contains("failgate"),
        "gate failure body must use malvin tag, got: {stderr:?}"
    );
    assert!(
        !stderr.contains(&error_tag) && !stderr.contains(&warning_tag),
        "gate failure must not use error or warning tags, got: {stderr:?}"
    );
}

#[cfg(unix)]
fn minimal_git_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path().to_path_buf();
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&work)
        .status()
        .expect("git init");
    std::fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = \"m\"\nversion = \"0.1.0\"\n",
    )
    .expect("Cargo.toml");
    crate::seed_malvin_checks(&work, "true\nlint check\n");
    (tmp, work)
}

#[cfg(unix)]
#[test]
fn gate_run_wires_private_runners_on_minimal_workspace() {
    let (_tmp, work) = minimal_git_workspace();
    let bin_dir = tempfile::tempdir().expect("bindir");
    install_zero_exit_gate_bins(bin_dir.path());
    let _guard = set_fake_command_dir(bin_dir.path());

    run_quality_gates_with_details(&work, RepoGateOutput::Tagged, None).expect("quality gates");
    run_repo_workspace_gates_with_details(&work, RepoGateOutput::Tagged, None)
        .expect("workspace gates");
}

#[test]
fn prefer_gate_outcome_over_checks_restore_keeps_gate_failure() {
    let gate = Err("__MALVIN_GATE_FAILURE__:`make lint` failed (exit 1)".into());
    let restore = Err("malvin_checks restore: blocked".into());
    let err = super::prefer_gate_outcome_over_checks_restore(gate, restore).unwrap_err();
    assert!(err.contains("make lint"));
    assert!(!err.contains("malvin_checks restore"));
}
