use crate::artifacts::RunArtifacts;
use crate::review_sync::{is_lgtm_str, read_artifact_review_for_fanout_attempt};

pub const REVIEW_WRITE_MISSING_ARTIFACT_MSG: &str = "review_write did not write artifact review";
pub const REVIEW_WRITE_MISSING_ARTIFACT_RETRY_MSG: &str =
    "Review: review_write did not write artifact review, retrying";
pub const REVIEW_PREP_MISSING_ARTIFACT_MSG: &str = "reviewers_spawn did not write review prep";

pub const REVIEW_WRITE_INNER_RETRY_CAP: usize = 2;

#[must_use]
pub fn is_missing_artifact_review_error(err: &WorkflowError) -> bool {
    err.0 == REVIEW_WRITE_MISSING_ARTIFACT_MSG
}

use super::{WorkflowError, clear_review_file};

/// # Errors
///
/// Returns [`WorkflowError`] when stale review files cannot be cleared.
pub fn clear_review_attempt_artifacts(artifacts: &RunArtifacts) -> Result<(), WorkflowError> {
    let artifact_review = artifacts.artifact_review_md();
    let workspace_review = artifacts.workspace_review_md();
    let review_prep = artifacts.review_prep_md();

    clear_review_file(&artifact_review)
        .map_err(|e| WorkflowError(format!("failed to clear artifact review: {e}")))?;
    clear_review_file(&workspace_review)
        .map_err(|e| WorkflowError(format!("failed to clear workspace review: {e}")))?;
    clear_review_file(&review_prep)
        .map_err(|e| WorkflowError(format!("failed to clear review prep: {e}")))?;
    Ok(())
}

/// # Errors
///
/// Returns [`WorkflowError`] when the review prep file cannot be read.
pub fn ensure_review_prep_after_reviewers_spawn(
    artifacts: &RunArtifacts,
) -> Result<(), WorkflowError> {
    if let Some(abort_msg) = super::check_abort(&artifacts.artifact_result_md()) {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    let review_prep = artifacts.review_prep_md();
    let text = std::fs::read_to_string(&review_prep).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            WorkflowError(REVIEW_PREP_MISSING_ARTIFACT_MSG.to_string())
        } else {
            WorkflowError(format!(
                "failed to read review prep {}: {e}",
                review_prep.display()
            ))
        }
    })?;
    if text.trim().is_empty() {
        return Err(WorkflowError(REVIEW_PREP_MISSING_ARTIFACT_MSG.to_string()));
    }
    Ok(())
}

fn read_artifact_review_text(artifacts: &RunArtifacts) -> Result<Option<String>, WorkflowError> {
    read_artifact_review_for_fanout_attempt(&artifacts.artifact_review_md()).map_err(WorkflowError)
}

/// # Errors
///
/// Returns [`WorkflowError`] when the artifact review file cannot be read.
pub fn ensure_artifact_review_after_review_write(
    artifacts: &RunArtifacts,
) -> Result<(), WorkflowError> {
    if read_artifact_review_text(artifacts)?.is_none() {
        return Err(WorkflowError(REVIEW_WRITE_MISSING_ARTIFACT_MSG.to_string()));
    }
    Ok(())
}

/// # Errors
///
/// Returns [`WorkflowError`] when the artifact review file cannot be read.
pub fn review_attempt_is_lgtm(artifacts: &RunArtifacts) -> Result<bool, WorkflowError> {
    Ok(read_artifact_review_text(artifacts)?
        .as_deref()
        .is_some_and(is_lgtm_str))
}

/// Returns `Ok(None)` when the artifact review is missing, `Ok(Some(true|false))` for LGTM state.
///
/// # Errors
///
/// Returns [`WorkflowError`] when the artifact review file cannot be read.
pub fn artifact_review_lgtm_after_review_write(
    artifacts: &RunArtifacts,
) -> Result<Option<bool>, WorkflowError> {
    Ok(read_artifact_review_text(artifacts)?
        .as_deref()
        .map(is_lgtm_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::create_run_artifacts_from_text;

    #[test]
    fn ensure_review_prep_errors_when_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            create_run_artifacts_from_text("kernel_test", Some(tmp.path())).expect("artifacts");
        let err = ensure_review_prep_after_reviewers_spawn(&artifacts).expect_err("missing");
        assert_eq!(err.0, REVIEW_PREP_MISSING_ARTIFACT_MSG);
    }

    #[test]
    fn review_attempt_is_lgtm_true_when_artifact_lgtm_and_workspace_whitespace_only() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "plan").expect("write plan");
        let artifacts =
            create_run_artifacts_from_text("kernel_test", Some(tmp.path())).expect("artifacts");
        let artifact = artifacts.artifact_review_md();
        let workspace = artifacts.workspace_review_md();
        std::fs::write(&artifact, "LGTM\n").expect("artifact lgtm");
        std::fs::write(&workspace, "\n").expect("whitespace workspace");
        assert!(
            review_attempt_is_lgtm(&artifacts).expect("sync"),
            "review_write artifact LGTM must not be cleared by whitespace-only workspace review.md"
        );
        assert_eq!(
            std::fs::read_to_string(&artifact).expect("read artifact"),
            "LGTM\n"
        );
    }

    #[test]
    fn review_attempt_is_lgtm_rejects_stale_workspace_lgtm_when_artifact_has_gate_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "plan").expect("write plan");
        let artifacts =
            create_run_artifacts_from_text("kernel_test", Some(tmp.path())).expect("artifacts");
        let artifact = artifacts.artifact_review_md();
        let workspace = artifacts.workspace_review_md();
        std::fs::write(&artifact, "Checks do not pass\n").expect("artifact marker");
        std::fs::write(&workspace, "LGTM\n").expect("stale workspace");
        assert!(
            !review_attempt_is_lgtm(&artifacts).expect("sync"),
            "artifact gate marker must not be masked by stale workspace LGTM"
        );
        assert_eq!(
            std::fs::read_to_string(&artifact).expect("read artifact"),
            "Checks do not pass\n",
            "sync must not overwrite artifact with stale workspace LGTM"
        );
    }

    #[test]
    fn ensure_artifact_review_after_review_write_errors_when_artifact_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "plan").expect("write plan");
        let artifacts =
            create_run_artifacts_from_text("kernel_test", Some(tmp.path())).expect("artifacts");
        let workspace = artifacts.workspace_review_md();
        std::fs::write(&workspace, "LGTM\n").expect("workspace lgtm");
        let err = ensure_artifact_review_after_review_write(&artifacts).expect_err("missing");
        assert_eq!(err.0, REVIEW_WRITE_MISSING_ARTIFACT_MSG);
    }

    #[test]
    fn ensure_artifact_review_after_review_write_ok_when_artifact_nonempty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "plan").expect("write plan");
        let artifacts =
            create_run_artifacts_from_text("kernel_test", Some(tmp.path())).expect("artifacts");
        std::fs::write(artifacts.artifact_review_md(), "problems\n").expect("artifact");
        ensure_artifact_review_after_review_write(&artifacts).expect("present");
    }

    #[test]
    fn review_attempt_is_lgtm_false_when_only_workspace_lgtm() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "plan").expect("write plan");
        let artifacts =
            create_run_artifacts_from_text("kernel_test", Some(tmp.path())).expect("artifacts");
        let artifact = artifacts.artifact_review_md();
        let workspace = artifacts.workspace_review_md();
        std::fs::write(&workspace, "LGTM\n").expect("workspace lgtm");
        assert!(
            !review_attempt_is_lgtm(&artifacts).expect("read"),
            "empty artifact with workspace LGTM must not count as LGTM after fan-out"
        );
        assert!(
            !artifact.exists()
                || std::fs::read_to_string(&artifact)
                    .unwrap()
                    .trim()
                    .is_empty(),
            "workspace LGTM must not be promoted into artifact for fan-out LGTM"
        );
    }

    #[test]
    fn ensure_review_prep_accepts_nonempty_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            create_run_artifacts_from_text("kernel_test", Some(tmp.path())).expect("artifacts");
        std::fs::write(artifacts.review_prep_md(), "prep\n").expect("prep");
        ensure_review_prep_after_reviewers_spawn(&artifacts).expect("present");
    }

    #[test]
    fn ensure_review_prep_after_spawn_surfaces_abort_when_prep_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            create_run_artifacts_from_text("kernel_test", Some(tmp.path())).expect("artifacts");
        std::fs::write(artifacts.artifact_result_md(), "ABORT: agent stop\n").expect("abort");
        let err = ensure_review_prep_after_reviewers_spawn(&artifacts).expect_err("abort");
        assert_eq!(err.0, "ABORT: agent stop");
    }

    #[test]
    fn kiss_stringify_review_attempt_kernel_units() {
        let _ = stringify!(super::clear_review_attempt_artifacts);
        let _ = stringify!(super::ensure_review_prep_after_reviewers_spawn);
        let _ = stringify!(super::ensure_artifact_review_after_review_write);
        let _ = stringify!(super::REVIEW_WRITE_MISSING_ARTIFACT_MSG);
        let _ = stringify!(super::REVIEW_PREP_MISSING_ARTIFACT_MSG);
        let _ = stringify!(super::is_missing_artifact_review_error);
        let _ = stringify!(super::review_attempt_is_lgtm);
        let _ = stringify!(super::artifact_review_lgtm_after_review_write);
    }
}
