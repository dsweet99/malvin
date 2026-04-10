use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::RunArtifacts;
pub(super) use crate::review_sync::{is_lgtm, sync_review_file};

pub(super) fn workflow_context(artifacts: &RunArtifacts) -> HashMap<String, String> {
    let mut context = HashMap::new();
    context.insert(
        "plan_path".to_string(),
        artifacts.plan_path.display().to_string(),
    );
    let kpop = artifacts
        .run_dir
        .join("_kpop")
        .canonicalize()
        .unwrap_or_else(|_| artifacts.run_dir.join("_kpop"));
    context.insert(
        "kpop_log_dir".to_string(),
        format_prompt_path(&kpop, &artifacts.work_dir),
    );
    context
}

pub(super) fn clear_review_file(p: &Path) {
    let _ = std::fs::remove_file(p);
}

/// Stem used in log name segments for **both** coder prompts (`implement.md`, …) and reviewer prompts (`review_1.md`, …).
/// Strips a trailing `.md` when present (case-sensitive); otherwise returns `filename` unchanged. Avoids panics on short names.
#[must_use]
pub(super) fn prompt_md_stem(filename: &str) -> &str {
    filename.strip_suffix(".md").unwrap_or(filename)
}

pub(super) fn format_prompt_path(path: &Path, base_dir: &Path) -> String {
    let path_r = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let base_r = base_dir.canonicalize().unwrap_or_else(|_| base_dir.to_path_buf());
    path_r.strip_prefix(&base_r).map_or_else(
        |_| path.display().to_string(),
        |r| format!("./{}", r.display()),
    )
}
