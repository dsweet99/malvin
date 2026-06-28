//! Integration tests for `malvin init`.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::InitOk;
use common::{git_stdout, malvin_init_output};

struct PreCommitMissingFixture {
    _path_root: tempfile::TempDir,
    isolated_bin: PathBuf,
    project: tempfile::TempDir,
}

fn pre_commit_missing_fixture() -> PreCommitMissingFixture {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();
    let git_stub = isolated_bin.join("git");
    std::fs::write(
        &git_stub,
        "#!/bin/sh\ncase \"$1\" in init) exit 0 ;; rev-parse) pwd ;; *) exit 0 ;; esac\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&git_stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&git_stub, perms).unwrap();
    }
    PreCommitMissingFixture {
        _path_root: path_root,
        isolated_bin,
        project: tempfile::tempdir().unwrap(),
    }
}

fn assert_pre_commit_missing_message(msg: &str) {
    assert!(msg.contains("Command:"), "expected startup command prelude: {msg:?}");
    assert!(msg.contains("Logs:"), "expected startup Logs header: {msg:?}");
    assert!(
        msg.contains("pre-commit"),
        "expected explicit pre-commit hint; got: {msg:?}"
    );
}

fn malvin_init_output_for(path: &Path, isolated_bin: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("PATH", isolated_bin)
        .args(["init", "python", "--path"])
        .arg(path)
        .output()
        .expect("spawn malvin init")
}

#[test]
fn malvin_init_fails_fast_when_pre_commit_missing_from_path() {
    let fixture = pre_commit_missing_fixture();
    let out = malvin_init_output_for(fixture.project.path(), &fixture.isolated_bin);

    assert!(
        !out.status.success(),
        "expected non-zero exit; stdout/stderr: {out:?}"
    );
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_pre_commit_missing_message(&msg);
}

#[test]
fn malvin_init_rejects_unknown_language() {
    let project = tempfile::tempdir().unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "javascript", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");

    assert!(!out.status.success(), "should reject unknown language");
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        msg.contains("Unknown language"),
        "should mention unknown language: {msg:?}"
    );
}

#[test]
fn malvin_init_rejects_no_languages() {
    let project = tempfile::tempdir().unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");

    assert!(!out.status.success(), "should reject missing language args");
}

#[test]
fn malvin_init_creates_expected_files_for_python_only() {
    let w = InitOk::new(&["python"]);
    let pre_commit = w.read_rel(".pre-commit-config.yaml");
    assert!(
        pre_commit.contains("ruff"),
        "python-only should have ruff hook"
    );
    assert!(
        !pre_commit.contains("clippy"),
        "python-only should not have clippy hook"
    );
    assert!(pre_commit.contains("kiss"), "should always have kiss hook");
    assert!(
        pre_commit.contains("check-untracked"),
        "should always have check-untracked hook"
    );


    assert!(w.path().join(".gitignore").exists());
    assert!(w.path().join(".kissignore").exists());
    assert!(w.path().join("admin/check_untracked.sh").exists());
}

#[test]
fn malvin_init_creates_expected_files_for_rust_only() {
    let w = InitOk::new(&["rust"]);
    let pre_commit = w.read_rel(".pre-commit-config.yaml");
    assert!(
        !pre_commit.contains("ruff"),
        "rust-only should not have ruff hook"
    );
    assert!(
        pre_commit.contains("clippy"),
        "rust-only should have clippy hook"
    );
}

#[test]
fn malvin_init_creates_expected_files_for_both_languages() {
    let w = InitOk::new(&["python", "rust"]);
    let pre_commit = w.read_rel(".pre-commit-config.yaml");
    assert!(
        pre_commit.contains("ruff"),
        "both languages should have ruff hook"
    );
    assert!(
        pre_commit.contains("clippy"),
        "both languages should have clippy hook"
    );
}

#[test]
fn malvin_init_language_args_are_case_insensitive() {
    let project = tempfile::tempdir().unwrap();
    let (out, _home) = malvin_init_output(project.path(), &["PYTHON", "Rust"]);
    assert!(
        out.status.success(),
        "malvin init with mixed case should succeed: {out:?}"
    );
}

#[test]
fn malvin_init_git_ls_tree_head_lists_expected_paths() {
    let w = InitOk::new(&["python"]);
    let tree = git_stdout(w.path(), &["ls-tree", "-r", "--name-only", "HEAD"]);
    assert!(tree.contains(".gitignore"));
}
