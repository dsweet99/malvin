//! Git plumbing: temp index, `write-tree`, `show`, `diff --name-status -z`.

use std::path::Path;
use std::process::Command;

use crate::edit_efficiency::error::EditEfficiencyError;

fn git_output(
    repo_root: &Path,
    index_path: Option<&Path>,
    args: &[&str],
    context: &str,
) -> Result<Vec<u8>, EditEfficiencyError> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo_root).args(args);
    if let Some(idx) = index_path {
        cmd.env("GIT_INDEX_FILE", idx);
    }
    let out = cmd.output().map_err(|e| EditEfficiencyError::GitCommand {
        context: context.to_string(),
        source: e,
    })?;
    if !out.status.success() {
        return Err(EditEfficiencyError::GitFailed {
            status: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    Ok(out.stdout)
}

fn git_output_ok(
    repo_root: &Path,
    index_path: Option<&Path>,
    args: &[&str],
    context: &str,
) -> Result<String, EditEfficiencyError> {
    let bytes = git_output(repo_root, index_path, args, context)?;
    String::from_utf8(bytes).map_err(|_| EditEfficiencyError::Utf8 {
        context: context.to_string(),
    })
}

/// Stage the full working tree into the temporary index and return a tree hash.
///
/// On the **first** use for a new temp index, `index_path` must not exist (do not create an empty
/// file: git rejects a zero-byte index). After git has created the index, later calls reuse the
/// same path and only run `git add -A` / `git write-tree`.
pub fn write_tree_from_worktree(
    repo_root: &Path,
    index_path: &Path,
) -> Result<String, EditEfficiencyError> {
    let _ = git_output(
        repo_root,
        Some(index_path),
        &["add", "-A"],
        "git add -A (temp index)",
    )?;
    let tree = git_output_ok(
        repo_root,
        Some(index_path),
        &["write-tree"],
        "git write-tree",
    )?;
    Ok(tree.trim().to_string())
}

fn blob_missing_in_tree_stderr(stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();
    s.contains("does not exist")
        || (s.contains("path") && s.contains("not in"))
        || s.contains("did not match any file(s) known to git")
}

/// Read blob bytes at `path` in `tree`.
///
/// Returns empty bytes when the path is absent from that tree (normal for add/delete sides).
/// Propagates [`EditEfficiencyError::GitFailed`] when `git show` fails for other reasons (permissions,
/// corrupt object database, invalid `tree` id, etc.).
pub fn blob_at_tree(
    repo_root: &Path,
    tree: &str,
    path: &str,
) -> Result<Vec<u8>, EditEfficiencyError> {
    let spec = format!("{tree}:{path}");
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo_root).arg("show").arg(&spec);
    let out = cmd.output().map_err(|e| EditEfficiencyError::GitCommand {
        context: format!("git show {spec}"),
        source: e,
    })?;
    if out.status.success() {
        return Ok(out.stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    if blob_missing_in_tree_stderr(&stderr) {
        return Ok(Vec::new());
    }
    Err(EditEfficiencyError::GitFailed {
        status: out.status.code().unwrap_or(-1),
        stderr: stderr.into_owned(),
    })
}

/// Raw `git diff --name-status -z -M` between two tree objects.
pub fn diff_name_status_z(
    repo_root: &Path,
    old_tree: &str,
    new_tree: &str,
) -> Result<Vec<u8>, EditEfficiencyError> {
    git_output(
        repo_root,
        None,
        &[
            "diff",
            "--name-status",
            "-z",
            "-M",
            old_tree,
            new_tree,
        ],
        "git diff --name-status -z -M",
    )
}

/// Ensure `repo_root` exists and looks like a git work tree.
pub fn validate_repo_root(repo_root: &Path) -> Result<(), EditEfficiencyError> {
    let git_dir = repo_root.join(".git");
    if !(repo_root.is_dir() && git_dir.exists()) {
        return Err(EditEfficiencyError::InvalidRepo(repo_root.to_path_buf()));
    }
    Ok(())
}

#[cfg(test)]
mod kiss_stringify {
    #[test]
    fn kiss_stringify_git_tree() {
        let _ = stringify!(super::git_output);
        let _ = stringify!(super::git_output_ok);
        let _ = stringify!(super::write_tree_from_worktree);
        let _ = stringify!(super::blob_missing_in_tree_stderr);
        let _ = stringify!(super::blob_at_tree);
        let _ = stringify!(super::diff_name_status_z);
        let _ = stringify!(super::validate_repo_root);
    }
}

