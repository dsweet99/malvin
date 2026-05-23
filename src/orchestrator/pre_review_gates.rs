use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::repo_checks::{RepoGateCommandFailure, RepoGateFailure, RepoGateOutput, run_repo_workspace_gates_with_details};

use super::WorkflowError;

fn write_review_bytes(review_path: &Path, content: &[u8]) -> Result<(), WorkflowError> {
    if let Some(parent) = review_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            WorkflowError(format!(
                "failed to create parent dirs for {}: {e}",
                review_path.display()
            ))
        })?;
    }
    std::fs::write(review_path, content).map_err(|e| {
        WorkflowError(format!(
            "failed to write pre-review gate review {}: {e}",
            review_path.display()
        ))
    })
}

/// Markdown body for `review.md` when pre-review quality gates fail.
#[must_use]
pub fn format_pre_review_gate_failure_review(
    failure: &RepoGateCommandFailure,
    quality_gates_log: &str,
) -> String {
    let exit = failure
        .exit_code
        .map_or_else(|| "signal".to_string(), |code| code.to_string());
    let mut body = format!(
        "Quality gates did not pass.\n\ncommand: {}\nexit code: {}\nfull output log: {}\n",
        failure.command, exit, quality_gates_log
    );
    if !failure.stdout.is_empty() {
        body.push_str("\nstdout:\n");
        body.push_str(&failure.stdout);
        if !failure.stdout.ends_with('\n') {
            body.push('\n');
        }
    }
    if !failure.stderr.is_empty() {
        body.push_str("\nstderr:\n");
        body.push_str(&failure.stderr);
        if !failure.stderr.ends_with('\n') {
            body.push('\n');
        }
    }
    body
}

pub fn write_pre_review_gate_failure_for_artifacts(
    artifacts: &RunArtifacts,
    failure: &RepoGateCommandFailure,
    quality_gates_log: &str,
) -> Result<(), WorkflowError> {
    let body = format_pre_review_gate_failure_review(failure, quality_gates_log);
    write_review_bytes(&artifacts.artifact_review_md(), body.as_bytes())?;
    write_review_bytes(&artifacts.workspace_review_md(), body.as_bytes())
}

/// Run workspace quality gates before a code review attempt.
///
/// Returns `Ok(())` when gates pass. On command failure, writes gate output to `review.md` and
/// returns `Err(RepoGateFailure::Command(..))` for the caller to run concerns.
pub fn run_pre_review_workspace_gates(
    artifacts: &RunArtifacts,
) -> Result<(), RepoGateFailure> {
    run_repo_workspace_gates_with_details(
        &artifacts.work_dir,
        RepoGateOutput::Tagged,
        Some(&artifacts.run_dir),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::create_run_artifacts_from_text;

    #[test]
    fn format_pre_review_gate_failure_includes_command_and_streams() {
        let failure = RepoGateCommandFailure {
            command: "kiss check".into(),
            exit_code: Some(1),
            stdout: "out line\n".into(),
            stderr: "err line\n".into(),
        };
        let body = format_pre_review_gate_failure_review(&failure, "./_malvin/run/q.log");
        assert!(body.contains("Quality gates did not pass"));
        assert!(body.contains("command: kiss check"));
        assert!(body.contains("exit code: 1"));
        assert!(body.contains("./_malvin/run/q.log"));
        assert!(body.contains("stdout:\nout line"));
        assert!(body.contains("stderr:\nerr line"));
    }

    #[test]
    fn write_pre_review_gate_failure_writes_artifact_and_workspace_review() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            create_run_artifacts_from_text("pre_review", Some(tmp.path())).expect("artifacts");
        let failure = RepoGateCommandFailure {
            command: "ruff check".into(),
            exit_code: Some(2),
            stdout: String::new(),
            stderr: "bad\n".into(),
        };
        write_pre_review_gate_failure_for_artifacts(&artifacts, &failure, "./log")
            .expect("write");
        let artifact = std::fs::read_to_string(artifacts.artifact_review_md()).expect("artifact");
        let workspace = std::fs::read_to_string(artifacts.workspace_review_md()).expect("workspace");
        assert_eq!(artifact, workspace);
        assert!(artifact.contains("ruff check"));
        assert!(artifact.contains("stderr:\nbad"));
    }
}
