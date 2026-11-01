use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const RECENT_RUN_DIRS: usize = 5;
pub const MAX_NON_GIT_UPLOAD_BYTES: u64 = 1 << 30;
const NON_GIT_EXCLUDES: &[&str] = &[
    "--exclude=./.git",
    "--exclude=./target",
    "--exclude=./node_modules",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitPlacement {
    pub toplevel: PathBuf,
    pub prefix: String,
}

fn git_stdout(dir: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status.success().then_some(out.stdout)
}

#[must_use]
pub fn git_placement(dir: &Path) -> Option<GitPlacement> {
    let top = git_stdout(dir, &["rev-parse", "--show-toplevel"])?;
    let prefix = git_stdout(dir, &["rev-parse", "--show-prefix"])?;
    Some(GitPlacement {
        toplevel: PathBuf::from(String::from_utf8_lossy(&top).trim()),
        prefix: String::from_utf8_lossy(&prefix).trim().to_string(),
    })
}

fn inside_git_work_tree(dir: &Path) -> Result<bool, String> {
    let answer = git_stdout(dir, &["rev-parse", "--is-inside-work-tree"]);
    if answer.is_some_and(|out| out.trim_ascii() == b"true") {
        return Ok(true);
    }
    let git_root = dir
        .ancestors()
        .find(|ancestor| ancestor.join(".git").symlink_metadata().is_ok());
    git_root.map_or(Ok(false), |root| {
        Err(format!(
            "{} has git metadata, but git could not read it as a work tree, so `--remote` refuses to upload {} because gitignored files could be included; run `git status` there to see the problem",
            root.display(),
            dir.display()
        ))
    })
}

pub fn selected_files(dir: &Path) -> Result<Vec<u8>, String> {
    let listed = git_stdout(dir, &["ls-files", "-co", "--exclude-standard", "-z"])
        .ok_or_else(|| {
            format!(
                "git could not list the files in {}, so `--remote` refuses to upload it because gitignored files could be included; run `git status` there to see the problem",
                dir.display()
            )
        })?;
    let mut kept = Vec::with_capacity(listed.len());
    for name in listed.split(|b| *b == 0).filter(|n| !n.is_empty()) {
        if dir.join(OsStr::from_bytes(name)).symlink_metadata().is_ok() {
            kept.extend_from_slice(name);
            kept.push(0);
        }
    }
    Ok(kept)
}

fn tar_from_list(dir: &Path, list: &[u8], out: &Path) -> Result<(), String> {
    let mut child = Command::new("tar")
        .arg("-czf")
        .arg(out)
        .args(["--null", "--no-recursion", "-T", "-"])
        .current_dir(dir)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run tar: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(list)
            .map_err(|e| format!("write tar file list: {e}"))?;
    }
    let status = child.wait().map_err(|e| format!("wait for tar: {e}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("tar failed packing {} ({status})", dir.display()))
}

fn run_tar(args: &[&str], dir: &Path, out: &Path) -> Result<(), String> {
    let status = Command::new("tar")
        .arg("-czf")
        .arg(out)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| format!("failed to run tar: {e}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("tar failed packing {} ({status})", dir.display()))
}

pub fn check_upload_size(dir: &Path, cap_bytes: u64) -> Result<(), String> {
    if inside_git_work_tree(dir)? {
        return Ok(());
    }
    let mut total = 0;
    if !tree_exceeds(dir, true, cap_bytes, &mut total) {
        return Ok(());
    }
    Err(format!(
        "{} is not in a git work tree, so `--remote` would upload everything under it, which is more than {} MiB (not counting .git, target, and node_modules); run from a git work tree or a smaller directory",
        dir.display(),
        cap_bytes >> 20
    ))
}

fn tree_exceeds(dir: &Path, top: bool, cap: u64, total: &mut u64) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let excluded = top
            && NON_GIT_EXCLUDES
                .iter()
                .any(|x| x.strip_prefix("--exclude=./") == entry.file_name().to_str());
        let Ok(meta) = entry.path().symlink_metadata() else {
            continue;
        };
        if excluded {
            continue;
        }
        if meta.is_dir() && tree_exceeds(&entry.path(), false, cap, total) {
            return true;
        }
        *total = total.saturating_add(meta.len());
        if *total > cap {
            return true;
        }
    }
    false
}

pub fn pack_workspace(dir: &Path, out: &Path) -> Result<bool, String> {
    if inside_git_work_tree(dir)? {
        tar_from_list(dir, &selected_files(dir)?, out)?;
        return Ok(true);
    }
    let mut args = NON_GIT_EXCLUDES.to_vec();
    args.push(".");
    run_tar(&args, dir, out)?;
    Ok(false)
}

#[must_use]
pub fn recent_run_dirs(logs_root: &Path, count: usize) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(logs_root) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.as_bytes().first().is_some_and(u8::is_ascii_digit))
        .collect();
    names.sort_unstable_by(|a, b| b.cmp(a));
    names.truncate(count);
    names
}

pub fn pack_logs(logs_root: &Path, names: &[String], out: &Path) -> Result<bool, String> {
    if names.is_empty() {
        return Ok(false);
    }
    let args: Vec<&str> = names.iter().map(String::as_str).collect();
    run_tar(&args, logs_root, out)?;
    Ok(true)
}
