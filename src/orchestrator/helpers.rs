use std::collections::HashMap;
use std::path::Path;

pub use crate::workflow_context::{
    format_prompt_path, workflow_context, workflow_context_paths_only,
};
pub(crate) use crate::workflow_context::insert_formatted;

/// Removes a review file when it exists; succeeds when `p` is absent.
///
/// # Errors
///
/// Returns [`std::io::Error`] when removal fails for reasons other than [`NotFound`](std::io::ErrorKind::NotFound).
pub fn clear_review_file(p: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(p) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

#[must_use]
pub fn check_abort(result_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(result_path).ok()?;
    let text = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("ABORT:") {
            return Some(rest.trim_start().to_string());
        }
    }
    None
}

/// Stem used in log name segments for coder prompts (`check_plan.md`, `implement.md`, …) and review prompts (`review.md`, `review_write.md`, …).
/// Strips a trailing `.md` when present (case-sensitive); otherwise returns `filename` unchanged. Avoids panics on short names.
#[must_use]
pub(crate) fn prompt_md_stem(filename: &str) -> &str {
    filename.strip_suffix(".md").unwrap_or(filename)
}

#[must_use]
pub fn format_exp_log_relative(
    artifacts: &crate::artifacts::RunArtifacts,
    exp_log: &Path,
) -> String {
    crate::workflow_context::format_prompt_path(exp_log, &artifacts.work_dir)
}

#[cfg(test)]
mod helpers_kiss_inline {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn format_exp_log_relative_under_work_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join(".malvin/logs").join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        let exp_log = run_dir.join("exp.log");
        std::fs::write(&exp_log, "x").expect("write");
        let artifacts = crate::artifacts::RunArtifacts {
            run_dir: run_dir.clone(),
            plan_path: run_dir.join("plan.md"),
            work_dir: tmp.path().to_path_buf(),
        };
        let rel = format_exp_log_relative(&artifacts, &exp_log);
        assert!(rel.contains("exp.log"));
    }

    #[test]
    fn insert_artifact_paths_and_resolve_path_against_base() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join(".malvin/logs").join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        let plan_path = run_dir.join("plan.md");
        std::fs::write(&plan_path, "p").expect("plan");
        let artifacts = crate::artifacts::RunArtifacts {
            run_dir,
            plan_path: plan_path.clone(),
            work_dir: tmp.path().to_path_buf(),
        };
        let ctx = crate::workflow_context::workflow_context_paths_only(&artifacts, "code");
        assert!(ctx.contains_key("quality_gates_log"));
        let _ = format_prompt_path(&plan_path, &artifacts.work_dir);
    }
}

