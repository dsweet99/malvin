#[must_use]
pub fn workflow_context(artifacts: &RunArtifacts) -> HashMap<String, String> {
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
    context.insert(
        "review_path".to_string(),
        format_prompt_path(&artifacts.artifact_review_md(), &artifacts.work_dir),
    );
    context.insert(
        "result_path".to_string(),
        format_prompt_path(&artifacts.artifact_result_md(), &artifacts.work_dir),
    );
    context
}

pub(crate) fn clear_review_file(p: &Path) {
    let _ = std::fs::remove_file(p);
}

pub(crate) fn check_abort(result_path: &Path) -> Option<String> {
    std::fs::read_to_string(result_path)
        .ok()
        .filter(|content| content.contains("ABORT"))
}

/// Stem used in log name segments for **both** coder prompts (`implement.md`, …) and reviewer prompts (`review_1.md`, …).
/// Strips a trailing `.md` when present (case-sensitive); otherwise returns `filename` unchanged. Avoids panics on short names.
#[must_use]
pub(crate) fn prompt_md_stem(filename: &str) -> &str {
    filename.strip_suffix(".md").unwrap_or(filename)
}

pub(crate) fn format_prompt_path(path: &Path, base_dir: &Path) -> String {
    let path_r = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let base_r = base_dir.canonicalize().unwrap_or_else(|_| base_dir.to_path_buf());
    path_r.strip_prefix(&base_r).map_or_else(
        |_| path.display().to_string(),
        |r| format!("./{}", r.display()),
    )
}

