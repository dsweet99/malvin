//! `malvin init` — install templates and bootstrap local tooling.

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;

use malvin::env_path::lookup_bin_on_path;

const TPL_GITIGNORE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/gitignore"));
const TPL_KISSIGNORE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/kissignore"));
const TPL_PRE_COMMIT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/pre-commit-config.yaml"));
const TPL_GROUNDING: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/grounding.md"));
const ADMIN_CHECK_UNTRACKED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/admin/check_untracked.sh"));

/// Arguments for [`run_init`].
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Overwrite files installed from `default_repo/` and refresh `admin/check_untracked.sh`.
    #[arg(long, default_value_t = false)]
    pub force: bool,
    /// Target directory (defaults to the current working directory).
    pub path: Option<PathBuf>,
}

/// `--force` overwrites files installed from `default_repo/` and refreshes `admin/check_untracked.sh`.
pub fn run_init(path: Option<PathBuf>, force: bool) -> Result<(), String> {
    let root = resolve_init_root(path)?;
    write_init_templates(&root, force)?;
    bootstrap_repo_tooling(&root)
}

fn resolve_init_root(path: Option<PathBuf>) -> Result<PathBuf, String> {
    let root = path.map_or_else(
        || std::env::current_dir().map_err(|e| e.to_string()),
        Ok,
    )?;
    if !root.exists() {
        std::fs::create_dir_all(&root).map_err(|e| {
            format!(
                "init: create directory {}: {e}",
                root.display()
            )
        })?;
    }
    Ok(root)
}

fn write_init_templates(root: &Path, force: bool) -> Result<(), String> {
    write_text_file(&root.join(".gitignore"), TPL_GITIGNORE, force)?;
    write_text_file(&root.join(".kissignore"), TPL_KISSIGNORE, force)?;
    write_text_file(&root.join(".pre-commit-config.yaml"), TPL_PRE_COMMIT, force)?;
    write_text_file(&root.join("grounding.md"), TPL_GROUNDING, force)?;

    let admin_dir = root.join("admin");
    std::fs::create_dir_all(&admin_dir).map_err(|e| format!("init: mkdir admin: {e}"))?;
    write_shell_script(&admin_dir.join("check_untracked.sh"), ADMIN_CHECK_UNTRACKED, force)?;
    Ok(())
}

fn bootstrap_repo_tooling(root: &Path) -> Result<(), String> {
    // Match product plan: templates include `.pre-commit-config.yaml`, then hooks, then `kiss init`, then Git LFS.
    require_on_path(
        "pre-commit",
        "`pre-commit` is not installed or not on PATH; install pre-commit (for example `pip install pre-commit`) before running `malvin init`.",
    )?;
    run_command_expect_success(
        Command::new("pre-commit").arg("install").current_dir(root),
        "`pre-commit install` failed (is this directory a git repository?).",
    )?;

    require_on_path(
        "kiss",
        "`kiss` is not installed or not on PATH; install kiss before running `malvin init`.",
    )?;
    run_command_expect_success(
        Command::new("kiss").arg("init").current_dir(root),
        "`kiss init` failed.",
    )?;

    install_git_lfs(root)?;
    Ok(())
}

fn require_on_path(bin: &str, err: &str) -> Result<(), String> {
    if lookup_bin_on_path(bin).is_none() {
        return Err(err.to_string());
    }
    Ok(())
}

fn install_git_lfs(root: &Path) -> Result<(), String> {
    let status = Command::new("git")
        .args(["lfs", "version"])
        .current_dir(root)
        .status()
        .map_err(|_| {
            "`git lfs` is not available (Git LFS not installed or not on PATH). Install Git LFS so `git lfs version` succeeds."
                .to_string()
        })?;
    if !status.success() {
        return Err(
            "`git lfs version` failed. Install Git LFS and ensure it is on PATH before running `malvin init`."
                .to_string(),
        );
    }
    run_command_expect_success(
        Command::new("git").args(["lfs", "install"]).current_dir(root),
        "`git lfs install` failed.",
    )
}

fn run_command_expect_success(cmd: &mut Command, err: &str) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{err} ({e})"))?;
    if status.success() {
        Ok(())
    } else {
        Err(err.to_string())
    }
}

fn write_text_file(path: &Path, contents: &str, force: bool) -> Result<(), String> {
    if path.exists() && !force {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("init: mkdir {}: {e}", parent.display()))?;
    }
    std::fs::write(path, contents)
        .map_err(|e| format!("init: write {}: {e}", path.display()))?;
    Ok(())
}

fn write_shell_script(path: &Path, contents: &str, force: bool) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_templates_are_non_empty() {
        assert!(!TPL_GITIGNORE.trim().is_empty());
        assert!(!TPL_PRE_COMMIT.trim().is_empty());
        assert!(ADMIN_CHECK_UNTRACKED.contains("check_untracked"));
    }
}
