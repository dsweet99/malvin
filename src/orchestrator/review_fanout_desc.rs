use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::prompts::{self, PromptStore};

use super::constants::REVIEW_DESCRIPTIONS_FILE;
use super::WorkflowError;

#[must_use]
pub fn parse_review_description_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[must_use]
pub fn reviewers_attempt_dir(run_dir: &Path, attempt: usize) -> PathBuf {
    run_dir.join(format!("reviewers_attempt_{attempt}"))
}

#[must_use]
pub fn reviewer_output_filename(index: usize) -> String {
    format!("reviewer_{index:03}.md")
}

pub fn load_review_description_lines(
    store: &PromptStore,
) -> Result<Vec<String>, WorkflowError> {
    let text = store
        .prompt_text(REVIEW_DESCRIPTIONS_FILE)
        .map_err(|e| WorkflowError(e.0))?;
    Ok(parse_review_description_lines(&text))
}

#[cfg(test)]
pub fn embedded_review_description_job_count() -> usize {
    parse_review_description_lines(include_str!(
        "../../default_prompts/review_descriptions.md"
    ))
    .len()
}

pub fn fanout_reviewer_render_context(
    workflow_context: &HashMap<String, String>,
    review_description: String,
    subdir_formatted: &str,
    reviewer_filename: &str,
) -> HashMap<String, String> {
    let mut job_ctx = HashMap::with_capacity(workflow_context.len());
    for (key, value) in workflow_context {
        if key == "memories" {
            continue;
        }
        job_ctx.insert(key.clone(), value.clone());
    }
    job_ctx.insert("review_description".to_string(), review_description);
    job_ctx.insert("reviewers_subdir".to_string(), subdir_formatted.to_string());
    job_ctx.insert(
        "reviewer_filename".to_string(),
        reviewer_filename.to_string(),
    );
    job_ctx
}

pub fn expand_review_description_line(
    line: &str,
    context: &HashMap<String, String>,
) -> Result<String, WorkflowError> {
    let expanded = prompts::render_template(line, context);
    prompts::enforce_no_unresolved_braces(&expanded).map_err(|e| WorkflowError(e.0))?;
    Ok(expanded)
}

pub fn verify_reviewer_output_files(
    reviewers_subdir: &Path,
    job_count: usize,
) -> Result<(), WorkflowError> {
    for index in 1..=job_count {
        let path = reviewers_subdir.join(reviewer_output_filename(index));
        let text = std::fs::read_to_string(&path).map_err(|e| {
            let label = if e.kind() == std::io::ErrorKind::NotFound {
                "missing reviewer output"
            } else {
                "failed to read reviewer output"
            };
            WorkflowError(format!("{label} {}: {e}", path.display()))
        })?;
        if text.trim().is_empty() {
            return Err(WorkflowError(format!(
                "reviewer output {} is empty",
                path.display()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_review_description_lines_skips_blanks() {
        let lines = parse_review_description_lines("a\n\n  b  \n");
        assert_eq!(lines, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn reviewer_output_filename_is_zero_padded() {
        assert_eq!(reviewer_output_filename(1), "reviewer_001.md");
        assert_eq!(reviewer_output_filename(12), "reviewer_012.md");
    }

    #[test]
    fn expand_review_description_line_substitutes_plan_path() {
        let mut ctx = HashMap::new();
        ctx.insert("plan_path".to_string(), "/tmp/plan.md".to_string());
        let line = "Find inconsistencies with `{{ plan_path }}`.";
        let expanded = expand_review_description_line(line, &ctx).expect("expand");
        assert!(
            expanded.contains("/tmp/plan.md"),
            "expected plan_path in expanded line, got: {expanded}"
        );
        assert!(
            !expanded.contains("{{ plan_path }}"),
            "unexpanded placeholder remains: {expanded}"
        );
    }

    #[test]
    fn embedded_description_line_count_is_nonzero() {
        assert!(super::embedded_review_description_job_count() > 0);
    }

    #[test]
    fn kiss_stringify_verify_reviewer_output_files() {
        let _ = stringify!(super::verify_reviewer_output_files);
        let _ = stringify!(super::fanout_reviewer_render_context);
    }

    #[test]
    fn verify_reviewer_output_files_requires_all_outputs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = verify_reviewer_output_files(dir.path(), 2).expect_err("missing files");
        assert!(
            err.0.contains("missing reviewer output"),
            "unexpected error: {}",
            err.0
        );
    }

    #[test]
    fn verify_reviewer_output_files_distinguishes_missing_from_read_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(reviewer_output_filename(1));
        std::fs::write(&path, "ok\n").expect("write reviewer file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000))
                .expect("chmod reviewer file");
            let err = verify_reviewer_output_files(dir.path(), 1).expect_err("unreadable file");
            assert!(
                err.0.contains("failed to read reviewer output"),
                "unexpected error: {}",
                err.0
            );
            assert!(
                !err.0.contains("missing reviewer output"),
                "read failure must not be reported as missing: {}",
                err.0
            );
        }
    }

    #[test]
    fn verify_reviewer_output_files_rejects_empty_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(reviewer_output_filename(1));
        std::fs::write(&path, "   \n").expect("write empty reviewer file");
        let err = verify_reviewer_output_files(dir.path(), 1).expect_err("empty file");
        assert!(
            err.0.contains("is empty"),
            "unexpected error: {}",
            err.0
        );
    }

    #[test]
    fn verify_reviewer_output_files_accepts_nonempty_outputs() {
        let dir = tempfile::tempdir().expect("tempdir");
        for index in 1..=3 {
            let path = dir.path().join(reviewer_output_filename(index));
            std::fs::write(&path, "tl;dr: ok\n").expect("write reviewer file");
        }
        verify_reviewer_output_files(dir.path(), 3).expect("all outputs present");
    }

    #[test]
    fn fanout_wave_count_matches_chunk_schedule_for_embedded_descriptions() {
        use crate::orchestrator::constants::{REVIEWER_FANOUT_CONCURRENCY, fanout_wave_count};

        let store = crate::prompts::PromptStore::default_store();
        let lines = load_review_description_lines(&store).expect("embedded descriptions");
        let n = lines.len();
        assert_eq!(fanout_wave_count(n), n.div_ceil(REVIEWER_FANOUT_CONCURRENCY));
        assert_eq!(
            lines.chunks(REVIEWER_FANOUT_CONCURRENCY).count(),
            fanout_wave_count(n)
        );
    }
}
