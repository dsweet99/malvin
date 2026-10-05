use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::workspace::{GitPlacement, git_placement};

pub const KEPT_PATCH_NAME: &str = "modal.patch";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchOutcome {
    Empty,
    Applied,
    Merged,
    Conflicted(PathBuf),
    Kept(PathBuf),
}

fn top_level_names(tar: &Path) -> Result<Vec<String>, String> {
    let listing = Command::new("tar")
        .arg("-tzf")
        .arg(tar)
        .output()
        .map_err(|e| format!("failed to run tar: {e}"))?;
    let mut tops: Vec<String> = String::from_utf8_lossy(&listing.stdout)
        .lines()
        .filter_map(|l| l.split('/').next().map(str::to_string))
        .filter(|n| !n.is_empty() && n != ".")
        .collect();
    tops.sort_unstable();
    tops.dedup();
    Ok(tops)
}

fn extract(tar: &Path, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| format!("mkdir {}: {e}", dest.display()))?;
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(tar)
        .arg("-C")
        .arg(dest)
        .status()
        .map_err(|e| format!("failed to run tar: {e}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("tar failed extracting run logs ({status})"))
}

pub fn import_logs(tar: &Path, logs_root: &Path) -> Result<Vec<PathBuf>, String> {
    if std::fs::metadata(tar).map_or(true, |m| m.len() == 0) {
        return Ok(Vec::new());
    }
    let tops = top_level_names(tar)?;
    extract(tar, logs_root)?;
    Ok(tops.into_iter().map(|n| logs_root.join(n)).collect())
}

fn git_apply(place: &GitPlacement, patch: &Path, three_way: bool) -> bool {
    let mut cmd = Command::new("git");
    cmd.arg("apply").arg("--whitespace=nowarn");
    if three_way {
        cmd.arg("--3way");
    }
    if !place.prefix.is_empty() {
        cmd.arg(format!("--directory={}", place.prefix.trim_end_matches('/')));
    }
    cmd.arg(patch)
        .current_dir(&place.toplevel)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn has_unmerged_paths(place: &GitPlacement) -> bool {
    Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(&place.toplevel)
        .output()
        .is_ok_and(|o| !o.stdout.is_empty())
}

fn keep_patch(patch: &Path, keep_dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(keep_dir).map_err(|e| format!("mkdir {}: {e}", keep_dir.display()))?;
    let kept = keep_dir.join(KEPT_PATCH_NAME);
    std::fs::copy(patch, &kept).map_err(|e| format!("copy patch to {}: {e}", kept.display()))?;
    Ok(kept)
}

pub fn apply_patch(patch: &Path, work_dir: &Path, keep_dir: &Path) -> Result<PatchOutcome, String> {
    if std::fs::metadata(patch).map_or(true, |m| m.len() == 0) {
        return Ok(PatchOutcome::Empty);
    }
    let Some(place) = git_placement(work_dir) else {
        return keep_patch(patch, keep_dir).map(PatchOutcome::Kept);
    };
    if git_apply(&place, patch, false) {
        return Ok(PatchOutcome::Applied);
    }
    let merged = git_apply(&place, patch, true);
    if merged {
        return Ok(PatchOutcome::Merged);
    }
    let kept = keep_patch(patch, keep_dir)?;
    if has_unmerged_paths(&place) {
        return Ok(PatchOutcome::Conflicted(kept));
    }
    Ok(PatchOutcome::Kept(kept))
}

#[must_use]
pub fn describe_outcome(outcome: &PatchOutcome) -> String {
    match outcome {
        PatchOutcome::Empty => "the remote run changed no files".to_string(),
        PatchOutcome::Applied => "applied the remote changes to the working tree".to_string(),
        PatchOutcome::Merged => {
            "applied the remote changes with a 3-way merge (they are staged in the index)".to_string()
        }
        PatchOutcome::Conflicted(p) => format!(
            "the remote changes conflict with local edits; conflict markers are in the working tree and the patch is at {}",
            p.display()
        ),
        PatchOutcome::Kept(p) => format!(
            "the remote changes were not applied; the patch is at {}",
            p.display()
        ),
    }
}
