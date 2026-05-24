use crate::artifacts::restore_workspace_session_dotfiles;
use crate::run_timing::TimingPhase;

use super::TidyAcpInput;
use super::TidyPromptRestore;

pub async fn run_tidy_prompt(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    kind: &str,
    phase: TimingPhase,
) -> Result<(), String> {
    input
        .client
        .run_coder_prompt(
            prompt,
            &input.artifacts.log_path(kind),
            kind,
            crate::acp::CoderPromptOptions {
                llm_phase: Some(phase),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub async fn run_tidy_prompt_with_restore(
    input: &mut TidyAcpInput<'_>,
    request: TidyPromptRestore<'_>,
) -> Result<(), String> {
    let acp_result = run_tidy_prompt(input, request.prompt, request.label, request.phase).await;
    let restore_result = restore_workspace_session_dotfiles(
        &input.artifacts.work_dir,
        request.session_dotfile_backups,
    )
    .map_err(|e| format!("tidy restore failed after {}: {e}", request.restore_context));
    match (acp_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
        (Err(e), Err(restore_error)) => Err(format!(
            "{e}; tidy restore failed after {}: {restore_error}",
            request.restore_context
        )),
    }
}

#[cfg(test)]
mod prompt_tests {
    use super::*;

    #[tokio::test]
    async fn run_tidy_prompt_errors_without_active_coder_session() {
        let mut client = crate::test_agent_client::smoke_agent_client();
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "tidy").expect("write plan");
        let artifacts =
            crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
        let store = crate::cli::tidy_flow::prepare_tidy_prompt_store().expect("store");
        let context =
            crate::workflow_context::workflow_context_paths_only(&artifacts, "tidy");
        let mut input = crate::cli::tidy_flow::TidyAcpInput {
            client: &mut client,
            artifacts: &artifacts,
            store: &store,
            context: &context,
            run_learn: false,
            quick: false,
        };
        let err = run_tidy_prompt(
            &mut input,
            "hello",
            "tidy",
            crate::run_timing::TimingPhase::Implement,
        )
        .await
        .expect_err("no session");
        assert!(!err.is_empty());
    }

    #[tokio::test]
    async fn run_tidy_prompt_with_restore_still_restores_after_prompt_failure() {
        let mut client = crate::test_agent_client::smoke_agent_client();
        let tmp = tempfile::tempdir().expect("tempdir");
        let checks = tmp.path().join(".malvin_checks");
        std::fs::write(&checks, "orig\n").expect("write checks");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "tidy").expect("write plan");
        let artifacts =
            crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
        let backups = crate::artifacts::SessionDotfileBackups::from_parts(
            crate::artifacts::backup_workspace_kissconfig_if_present(&artifacts.work_dir).unwrap(),
            crate::artifacts::backup_workspace_malvin_checks_if_present(&artifacts.work_dir)
                .unwrap(),
            crate::artifacts::backup_workspace_kissignore_if_present(&artifacts.work_dir).unwrap(),
        );
        std::fs::write(&checks, "mutated\n").expect("mutate checks");
        let store = crate::cli::tidy_flow::prepare_tidy_prompt_store().expect("store");
        let context =
            crate::workflow_context::workflow_context_paths_only(&artifacts, "tidy");
        let mut input = crate::cli::tidy_flow::TidyAcpInput {
            client: &mut client,
            artifacts: &artifacts,
            store: &store,
            context: &context,
            run_learn: false,
            quick: false,
        };
        let err = run_tidy_prompt_with_restore(
            &mut input,
            crate::cli::tidy_flow::TidyPromptRestore {
                prompt: "x",
                label: "tidy",
                phase: crate::run_timing::TimingPhase::Implement,
                session_dotfile_backups: &backups,
                restore_context: "tidy",
            },
        )
        .await
        .expect_err("prompt fails");
        assert!(!err.is_empty());
        assert_eq!(
            std::fs::read_to_string(&checks).expect("read checks"),
            "orig\n"
        );
    }
}
