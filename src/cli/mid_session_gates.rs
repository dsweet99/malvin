use std::future::Future;
use std::pin::Pin;

use malvin::acp::AgentClient;
use malvin::artifacts::{GroundingBackup, RunArtifacts};

use super::repo_checks::{
    RepoGateCommandFailure, RepoGateFailure, RepoGateOutput, run_repo_workspace_gates,
    run_repo_workspace_gates_with_details,
};
use super::tidy_flow::run_tidy_prompt_after_post_run_gate_failure;

pub(super) async fn pre_summary_repo_gates_tidy_retry_flow<F, G, Fut, S>(
    first_gates: F,
    on_command_failure: G,
    second_gates: S,
) -> Result<(), String>
where
    F: FnOnce() -> Result<(), RepoGateFailure>,
    G: FnOnce(RepoGateCommandFailure) -> Fut,
    Fut: Future<Output = Result<(), String>>,
    S: FnOnce() -> Result<(), String>,
{
    match first_gates() {
        Ok(()) => Ok(()),
        Err(RepoGateFailure::Command(failure)) => {
            on_command_failure(failure).await?;
            second_gates()
                .map_err(|e| format!("post-run gates still failing after one tidy.md retry: {e}"))
        }
        Err(RepoGateFailure::Message(err)) => Err(err),
    }
}

pub fn mid_pre_summary_repo_gates<'a>(
    client: &'a mut AgentClient,
    artifacts: &'a RunArtifacts,
    grounding_backup: &'a GroundingBackup,
) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(run_pre_summary_repo_gates_with_tidy_retry(
        client,
        artifacts,
        grounding_backup,
    ))
}

pub async fn run_pre_summary_repo_gates_with_tidy_retry(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    grounding_backup: &GroundingBackup,
) -> Result<(), String> {
    let work_dir = artifacts.work_dir.clone();
    let run_dir = artifacts.run_dir.clone();
    pre_summary_repo_gates_tidy_retry_flow(
        || {
            run_repo_workspace_gates_with_details(
                &work_dir,
                RepoGateOutput::Tagged,
                Some(&run_dir),
            )
        },
        |failure| async move {
            run_tidy_prompt_after_post_run_gate_failure(
                client,
                artifacts,
                grounding_backup,
                &failure,
            )
            .await
        },
        || {
            run_repo_workspace_gates(
                &work_dir,
                RepoGateOutput::Tagged,
                Some(&run_dir),
            )
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::pre_summary_repo_gates_tidy_retry_flow;
    use crate::cli::repo_checks::{RepoGateCommandFailure, RepoGateFailure};

    #[tokio::test]
    async fn tidy_retry_flow_ok_when_first_gates_pass() {
        let first_calls = AtomicUsize::new(0);
        let tidy_calls = AtomicUsize::new(0);
        let second_calls = AtomicUsize::new(0);
        let out = pre_summary_repo_gates_tidy_retry_flow(
            || {
                first_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            |_failure| async {
                tidy_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            || {
                second_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .await;
        assert!(out.is_ok());
        assert_eq!(first_calls.load(Ordering::SeqCst), 1);
        assert_eq!(tidy_calls.load(Ordering::SeqCst), 0);
        assert_eq!(second_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn tidy_retry_flow_returns_message_without_tidy() {
        let tidy_calls = AtomicUsize::new(0);
        let err = pre_summary_repo_gates_tidy_retry_flow(
            || Err(RepoGateFailure::Message("no workspace".into())),
            |_failure| async {
                tidy_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            || Ok(()),
        )
        .await
        .expect_err("expected message error");
        assert_eq!(err, "no workspace");
        assert_eq!(tidy_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn tidy_retry_flow_runs_tidy_then_second_gates_on_command_failure() {
        let first_calls = AtomicUsize::new(0);
        let tidy_calls = Arc::new(AtomicUsize::new(0));
        let second_calls = AtomicUsize::new(0);
        let failure = RepoGateCommandFailure {
            command: "kiss check".into(),
            exit_code: Some(1),
            stdout: String::new(),
            stderr: String::new(),
        };
        let failure_for_first = failure.clone();
        let out = pre_summary_repo_gates_tidy_retry_flow(
            || {
                first_calls.fetch_add(1, Ordering::SeqCst);
                Err(RepoGateFailure::Command(failure_for_first))
            },
            |f| {
                let tc = Arc::clone(&tidy_calls);
                async move {
                    tc.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(f.command, "kiss check");
                    Ok(())
                }
            },
            || {
                second_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .await;
        assert!(out.is_ok());
        assert_eq!(first_calls.load(Ordering::SeqCst), 1);
        assert_eq!(tidy_calls.load(Ordering::SeqCst), 1);
        assert_eq!(second_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn tidy_retry_flow_wraps_second_gate_failure() {
        let err = pre_summary_repo_gates_tidy_retry_flow(
            || {
                Err(RepoGateFailure::Command(RepoGateCommandFailure {
                    command: "x".into(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                }))
            },
            |_f| async { Ok(()) },
            || Err("still broken".into()),
        )
        .await
        .expect_err("expected second gate failure");
        assert!(
            err.contains("post-run gates still failing after one tidy.md retry")
                && err.contains("still broken"),
            "unexpected err: {err}"
        );
    }

    #[tokio::test]
    async fn tidy_retry_flow_propagates_tidy_failure() {
        let err = pre_summary_repo_gates_tidy_retry_flow(
            || {
                Err(RepoGateFailure::Command(RepoGateCommandFailure {
                    command: "x".into(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                }))
            },
            |_f| async { Err("tidy session failed".into()) },
            || Ok(()),
        )
        .await
        .expect_err("expected tidy error");
        assert_eq!(err, "tidy session failed");
    }
}
