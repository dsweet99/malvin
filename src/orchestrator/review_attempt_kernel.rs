use std::collections::HashMap;
use std::path::PathBuf;

use crate::acp::AgentClient;
use crate::artifacts::RunArtifacts;
use crate::prompts::PromptStore;
use crate::review_sync::{is_lgtm_str, sync_review_file_for_attempt};

use super::review_fanout_desc::{
    load_review_description_lines, reviewers_attempt_dir, verify_reviewer_output_files,
};
use super::review_fanout_run::{FanoutPrepareInput, run_review_fanout_jobs};
use super::{WorkflowError, clear_review_file};

pub struct ReviewAttemptKernelInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub context: &'a HashMap<String, String>,
    pub descriptions: &'a [String],
    pub attempt: usize,
}

/// # Errors
///
/// Returns [`WorkflowError`] when descriptions cannot be loaded or the file has no lines.
pub fn load_review_descriptions_for_kernel(
    store: &PromptStore,
) -> Result<Vec<String>, WorkflowError> {
    let descriptions = load_review_description_lines(store)?;
    if descriptions.is_empty() {
        return Err(WorkflowError(
            "review_descriptions.md has no non-empty lines".to_string(),
        ));
    }
    Ok(descriptions)
}

/// # Errors
///
/// Returns [`WorkflowError`] when review files cannot be cleared, fan-out jobs fail, or outputs are missing.
pub async fn run_review_fanout_prefix(
    client: &AgentClient,
    input: &ReviewAttemptKernelInput<'_>,
) -> Result<PathBuf, WorkflowError> {
    let artifact_review = input.artifacts.artifact_review_md();
    let workspace_review = input.artifacts.workspace_review_md();

    clear_review_file(&artifact_review)
        .map_err(|e| WorkflowError(format!("failed to clear artifact review: {e}")))?;
    clear_review_file(&workspace_review)
        .map_err(|e| WorkflowError(format!("failed to clear workspace review: {e}")))?;

    let reviewers_subdir = reviewers_attempt_dir(&input.artifacts.run_dir, input.attempt);
    let fanout = FanoutPrepareInput {
        store: input.store,
        artifacts: input.artifacts,
        context: input.context,
        descriptions: input.descriptions,
        reviewers_subdir: &reviewers_subdir,
        attempt: input.attempt,
    };
    run_review_fanout_jobs(client, fanout).await?;
    verify_reviewer_output_files(&reviewers_subdir, input.descriptions.len())?;
    Ok(reviewers_subdir)
}

/// # Errors
///
/// Returns [`WorkflowError`] when review files cannot be synced.
pub fn review_attempt_is_lgtm(artifacts: &RunArtifacts) -> Result<bool, WorkflowError> {
    let artifact_review = artifacts.artifact_review_md();
    let workspace_review = artifacts.workspace_review_md();
    let review_text = sync_review_file_for_attempt(&artifact_review, &workspace_review)
        .map_err(WorkflowError)?;
    Ok(review_text.as_deref().is_some_and(is_lgtm_str))
}

#[cfg(test)]
mod tests {
    use super::super::review_fanout_desc::parse_review_description_lines;
    use crate::artifacts::create_run_artifacts_from_text;
    use crate::prompts::PromptStore;
    use crate::review_sync::sync_review_file_for_attempt;

    use super::*;

    #[test]
    fn load_review_descriptions_rejects_empty_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let prompts = dir.path().join("prompts");
        std::fs::create_dir_all(&prompts).expect("mkdir");
        std::fs::write(prompts.join("review_descriptions.md"), "\n  \n").expect("write");
        let store = PromptStore::with_root(prompts);
        let err = load_review_descriptions_for_kernel(&store).expect_err("empty");
        assert!(
            err.0.contains("no non-empty lines"),
            "unexpected: {}",
            err.0
        );
    }

    #[test]
    fn review_attempt_is_lgtm_true_when_artifact_lgtm_and_workspace_whitespace_only() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "plan").expect("write plan");
        let artifacts = create_run_artifacts_from_text("kernel_test", Some(tmp.path()))
            .expect("artifacts");
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
        let artifacts = create_run_artifacts_from_text("kernel_test", Some(tmp.path()))
            .expect("artifacts");
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
    fn review_attempt_is_lgtm_after_workspace_sync() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "plan").expect("write plan");
        let artifacts = create_run_artifacts_from_text("kernel_test", Some(tmp.path()))
            .expect("artifacts");
        let workspace = artifacts.workspace_review_md();
        std::fs::write(&workspace, "LGTM\n").expect("workspace lgtm");
        assert!(review_attempt_is_lgtm(&artifacts).expect("sync"));
        let synced = sync_review_file_for_attempt(
            &artifacts.artifact_review_md(),
            &workspace,
        )
        .expect("sync");
        assert!(synced.as_deref().is_some_and(is_lgtm_str));
    }

    #[test]
    fn embedded_review_description_line_count_matches_store() {
        let embedded = parse_review_description_lines(include_str!(
            "../../default_prompts/review_descriptions.md"
        ));
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let loaded = load_review_descriptions_for_kernel(&store).expect("load");
        assert_eq!(loaded.len(), embedded.len());
        assert!(!loaded.is_empty());
    }

    #[test]
    fn kiss_stringify_review_attempt_kernel_units() {
        let _ = stringify!(super::run_review_fanout_prefix);
        let _ = stringify!(super::review_attempt_is_lgtm);
        let _ = stringify!(super::load_review_descriptions_for_kernel);
        let _ = stringify!(super::ReviewAttemptKernelInput);
    }
}
