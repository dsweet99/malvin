use std::path::Path;
use std::process::Command;

use super::init_cmd_mid_core::run_command_expect_success;
use crate::require_kiss_for_malvin;

fn inside_git_work_tree(root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(root)
        .output()
        .is_ok_and(|o| {
            o.status.success()
                && String::from_utf8_lossy(&o.stdout).trim().eq_ignore_ascii_case("true")
        })
}

pub(super) fn ensure_git_repo(root: &Path) -> Result<(), String> {
    if inside_git_work_tree(root) {
        return Ok(());
    }
    run_command_expect_success(
        Command::new("git").arg("init").current_dir(root),
        "`git init` failed.",
    )?;
    let _ = Command::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(root)
        .status();
    Ok(())
}

fn pre_commit_hooks_installed(root: &Path) -> bool {
    let hook = root.join(".git/hooks/pre-commit");
    hook.is_file()
        && std::fs::read_to_string(&hook)
            .ok()
            .is_some_and(|body| body.contains("pre-commit"))
}

pub(super) fn ensure_pre_commit_hooks(root: &Path) -> Result<(), String> {
    if pre_commit_hooks_installed(root) {
        return Ok(());
    }
    run_command_expect_success(
        Command::new("pre-commit").arg("install").current_dir(root),
        "`pre-commit install` failed.",
    )
}

fn kiss_repo_initialized(root: &Path) -> bool {
    root.join(".kissconfig").is_file()
}

pub(super) fn ensure_kiss_repo_init(root: &Path) -> Result<(), String> {
    require_kiss_for_malvin("init")?;
    if kiss_repo_initialized(root) {
        return Ok(());
    }
    run_command_expect_success(
        Command::new("kiss").arg("init").current_dir(root),
        "`kiss init` failed.",
    )
}

fn git_lfs_hooks_installed(root: &Path) -> bool {
    Command::new("git")
        .args(["config", "--local", "--get", "lfs.repositoryformatversion"])
        .current_dir(root)
        .output()
        .is_ok_and(|o| o.status.success())
}

pub(super) fn ensure_git_lfs_hooks(root: &Path) -> Result<(), String> {
    let err = "`git lfs` is not available. Install Git LFS so `git lfs version` succeeds.";
    let status = Command::new("git")
        .args(["lfs", "version"])
        .current_dir(root)
        .status()
        .map_err(|_| err.to_string())?;
    if !status.success() {
        return Err(err.to_string());
    }
    if git_lfs_hooks_installed(root) {
        return Ok(());
    }
    run_command_expect_success(
        Command::new("git")
            .args(["lfs", "install"])
            .current_dir(root),
        "`git lfs install` failed.",
    )
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn inside_git_work_tree_false_without_repo() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!inside_git_work_tree(tmp.path()));
    }

    #[test]
    fn ensure_git_repo_creates_repo_on_main() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_git_repo(tmp.path()).unwrap();
        assert!(inside_git_work_tree(tmp.path()));
        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(branch.status.success());
        assert_eq!(
            String::from_utf8_lossy(&branch.stdout).trim(),
            "main"
        );
    }

    #[test]
    fn ensure_git_repo_skips_when_already_inside_work_tree() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_git_repo(tmp.path()).unwrap();
        ensure_git_repo(tmp.path()).unwrap();
    }

    #[test]
    fn pre_commit_hooks_installed_false_without_hook() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_git_repo(tmp.path()).unwrap();
        assert!(!pre_commit_hooks_installed(tmp.path()));
    }

    #[test]
    fn ensure_pre_commit_hooks_fails_without_git_repo() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".pre-commit-config.yaml"),
            "repos: []\n",
        )
        .unwrap();
        let err = ensure_pre_commit_hooks(tmp.path()).unwrap_err();
        assert!(
            err.contains("`pre-commit install` failed"),
            "expected pre-commit install failure without git; got: {err:?}"
        );
    }

    #[test]
    fn kiss_repo_initialized_false_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!kiss_repo_initialized(tmp.path()));
    }

    #[test]
    fn ensure_pre_commit_hooks_skips_when_hook_present() {
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        let hook = tmp.path().join(".git/hooks/pre-commit");
        std::fs::write(&hook, "#!/bin/sh\n# pre-commit stub\n").unwrap();
        ensure_pre_commit_hooks(tmp.path()).unwrap();
    }

    #[test]
    fn ensure_kiss_repo_init_skips_when_kissconfig_present() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".kissconfig"), "[python]\n").unwrap();
        ensure_kiss_repo_init(tmp.path()).unwrap();
    }

    #[test]
    fn ensure_git_lfs_hooks_succeeds_when_git_lfs_available() {
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        if Command::new("git")
            .args(["lfs", "version"])
            .status()
            .is_ok_and(|s| s.success())
        {
            ensure_git_lfs_hooks(tmp.path()).unwrap();
            assert!(git_lfs_hooks_installed(tmp.path()));
            ensure_git_lfs_hooks(tmp.path()).unwrap();
        }
    }
}
