use std::path::{Path, PathBuf};
use std::process::Command;

use super::{
    ADMIN_CHECK_UNTRACKED, HOOK_CLIPPY, HOOK_KISS, HOOK_RUFF, HOOK_UNTRACKED, Language,
    PRE_COMMIT_HEADER, TPL_GITIGNORE, TPL_KISSIGNORE,
};
use crate::{lookup_bin_on_path, require_kiss_for_malvin};

pub(super) fn build_pre_commit_config(languages: &[Language]) -> String {
    let mut config = PRE_COMMIT_HEADER.to_string();
    if languages.contains(&Language::Python) {
        config.push_str(HOOK_RUFF);
    }
    if languages.contains(&Language::Rust) {
        config.push_str(HOOK_CLIPPY);
    }
    config.push_str(HOOK_KISS);
    config.push_str(HOOK_UNTRACKED);
    config
}

pub(super) fn resolve_init_root(path: Option<PathBuf>) -> Result<PathBuf, String> {
    let root = path.map_or_else(|| std::env::current_dir().map_err(|e| e.to_string()), Ok)?;
    if !root.exists() {
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("init: create directory {}: {e}", root.display()))?;
    }
    Ok(root)
}

pub(super) fn write_init_templates(root: &Path, force: bool, languages: &[Language]) -> Result<(), String> {
    write_text_file(&root.join(".gitignore"), TPL_GITIGNORE, force)?;
    write_text_file(&root.join(".kissignore"), TPL_KISSIGNORE, force)?;
    let pre_commit_config = build_pre_commit_config(languages);
    write_text_file(
        &root.join(".pre-commit-config.yaml"),
        &pre_commit_config,
        force,
    )?;
    let admin_dir = root.join("admin");
    std::fs::create_dir_all(&admin_dir).map_err(|e| format!("init: mkdir admin: {e}"))?;
    write_shell_script(
        &admin_dir.join("check_untracked.sh"),
        ADMIN_CHECK_UNTRACKED,
        force,
    )?;
    Ok(())
}

pub(super) fn bootstrap_repo_tooling(root: &Path) -> Result<(), String> {
    require_on_path(
        "pre-commit",
        "`pre-commit` is not installed; run `pip install pre-commit`.",
    )?;
    run_command_expect_success(
        Command::new("pre-commit").arg("install").current_dir(root),
        "`pre-commit install` failed.",
    )?;
    require_kiss_for_malvin("init")?;
    run_command_expect_success(
        Command::new("kiss").arg("init").current_dir(root),
        "`kiss init` failed.",
    )?;
    install_git_lfs(root)?;
    create_initial_commit(root)
}

pub(super) fn create_initial_commit(root: &Path) -> Result<(), String> {
    if repo_already_has_commits(root) {
        return Ok(());
    }
    run_command_expect_success(
        Command::new("git").args(["add", "."]).current_dir(root),
        "`git add .` failed.",
    )?;
    let has_staged = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(root)
        .status()
        .is_ok_and(|s| !s.success());
    if has_staged {
        crate::output::print_stderr_line(
            crate::output::MALVIN_WHO,
            "init: creating initial commit (skipping pre-commit hooks to avoid bootstrap cycle)",
        );
        run_command_expect_success(
            Command::new("git")
                .args([
                    "-c",
                    "user.name=malvin",
                    "-c",
                    "user.email=malvin@localhost",
                ])
                .args([
                    "commit",
                    "--no-verify",
                    "-m",
                    "Initial commit from malvin init",
                ])
                .current_dir(root),
            "`git commit` failed.",
        )?;
        ensure_branch_is_main(root)?;
    }
    Ok(())
}

fn repo_already_has_commits(root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .is_ok_and(|o| o.status.success())
}

pub(super) fn ensure_branch_is_main(root: &Path) -> Result<(), String> {
    let current = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("`git branch --show-current` failed: {e}"))?;
    let branch = String::from_utf8_lossy(&current.stdout);
    if branch.trim() == "main" {
        return Ok(());
    }
    run_command_expect_success(
        Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(root),
        "`git branch -M main` failed.",
    )
}

pub(super) fn require_on_path(bin: &str, err: &str) -> Result<(), String> {
    if lookup_bin_on_path(bin).is_none() {
        return Err(err.to_string());
    }
    Ok(())
}

fn install_git_lfs(root: &Path) -> Result<(), String> {
    let err = "`git lfs` is not available. Install Git LFS so `git lfs version` succeeds.";
    let status = Command::new("git")
        .args(["lfs", "version"])
        .current_dir(root)
        .status()
        .map_err(|_| err.to_string())?;
    if !status.success() {
        return Err(err.to_string());
    }
    run_command_expect_success(
        Command::new("git")
            .args(["lfs", "install"])
            .current_dir(root),
        "`git lfs install` failed.",
    )
}

pub(super) fn run_command_expect_success(cmd: &mut Command, err: &str) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{err} ({e})"))?;
    if status.success() {
        Ok(())
    } else {
        Err(err.to_string())
    }
}

pub(super) fn write_text_file(path: &Path, contents: &str, force: bool) -> Result<(), String> {
    if path.exists() && !force {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("init: mkdir {}: {e}", parent.display()))?;
    }
    std::fs::write(path, contents).map_err(|e| format!("init: write {}: {e}", path.display()))
}

pub(super) fn write_shell_script(path: &Path, contents: &str, force: bool) -> Result<(), String> {
    if path.exists() && !force {
        return Ok(());
    }
    write_text_file(path, contents, force)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(|e| format!("init: stat {}: {e}", path.display()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)
            .map_err(|e| format!("init: chmod {}: {e}", path.display()))?;
    }
    Ok(())
}
