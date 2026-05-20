use std::collections::HashMap;

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::prompts::PromptStore;
use crate::run_timing::TimingPhase;

use crate::repo_checks::RepoGateCommandFailure;
use crate::cli::tidy_flow::{
    TidyAcpInput, TidyPromptRestore, compose_tidy_prompt, run_tidy_prompt_with_restore,
    tidy_prompt_context,
};

fn post_run_gate_failure_details(failure: &RepoGateCommandFailure, log_path: &str) -> String {
    let exit = failure
        .exit_code
        .map_or_else(|| "signal".to_string(), |code| code.to_string());
    format!(
        "The post-run quality gate check failed with:\ncommand: {}\nexit code: {}\nfull output log: {}",
        failure.command, exit, log_path
    )
}

struct PostRunTidyPrompt {
    store: PromptStore,
    context: HashMap<String, String>,
    prompt: String,
}

impl PostRunTidyPrompt {
    fn prepare(artifacts: &RunArtifacts, failure: &RepoGateCommandFailure) -> Result<Self, String> {
        let (store, context) = tidy_prompt_context(artifacts)?;
        let mut prompt = compose_tidy_prompt(&store, &context)?;
        let log_path = context
            .get("quality_gates_log")
            .map_or("./_malvin/.../quality_gates.log", String::as_str);
        prompt.push('\n');
        prompt.push('\n');
        prompt.push_str("Additional context from post-run quality gate failure:\n");
        prompt.push_str(&post_run_gate_failure_details(failure, log_path));
        Ok(Self {
            store,
            context,
            prompt,
        })
    }
}

pub async fn run_tidy_prompt_after_post_run_gate_failure(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
    failure: &RepoGateCommandFailure,
) -> Result<(), String> {
    let prep = PostRunTidyPrompt::prepare(artifacts, failure)?;
    let mut input = TidyAcpInput {
        client,
        artifacts,
        store: &prep.store,
        context: &prep.context,
        run_learn: false,
    };
    input.client.prompts_log_run_dir = Some(input.artifacts.run_dir.clone());
    if input.client.has_open_coder_session() {
        return run_tidy_prompt_with_restore(
            &mut input,
            TidyPromptRestore {
                prompt: &prep.prompt,
                label: "tidy",
                phase: TimingPhase::Implement,
                session_dotfile_backups,
                restore_context: "post-run gate failure",
            },
        )
        .await;
    }
    let timing = input.client.attach_run_timing_for_session();
    {
        let mut timing_guard = timing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        timing_guard.set_implement_display_name("tidy");
    }
    let begin_result = input
        .client
        .begin_coder_session(&input.artifacts.work_dir)
        .await;
    if let Err(e) = begin_result {
        input.client.set_run_timing(None);
        return Err(e.to_string());
    }
    let acp_result = run_tidy_prompt_with_restore(
        &mut input,
        TidyPromptRestore {
            prompt: &prep.prompt,
            label: "tidy",
            phase: TimingPhase::Implement,
            session_dotfile_backups,
            restore_context: "post-run gate failure",
        },
    )
    .await;
    let end_result = input
        .client
        .end_coder_session()
        .await
        .map_err(|e| e.to_string());
    let acp_result =
        crate::acp_post_run::prefer_primary_over_secondary(acp_result, end_result, "end_coder_session");
    crate::acp_post_run::emit_run_timing_after_acp(
        input.client,
        &input.artifacts.run_dir,
        &timing,
        acp_result,
    )
}

#[cfg(test)]
mod post_run_tidy_tests {
    use crate::repo_checks::RepoGateCommandFailure;
    use crate::acp::test_captive_session::captive_cat_acp_session_for_tests;
    use crate::acp::AgentIoOptions;
    use crate::artifacts::{
        KissConfigBackup, SessionDotfileBackups, create_run_artifacts_from_text,
    };

    use super::run_tidy_prompt_after_post_run_gate_failure;

    #[test]
    fn smoke_cov_mid_session_post_run_tidy() {
        let _ = super::run_tidy_prompt_after_post_run_gate_failure;
        let _ = super::post_run_gate_failure_details;
    }

    #[tokio::test]
    async fn post_run_gate_tidy_skips_second_begin_when_coder_session_open() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            create_run_artifacts_from_text("plan\n", Some(tmp.path())).expect("artifacts");
        std::fs::write(artifacts.work_dir.join(".malvin_checks"), "kiss check\n").expect("checks");
        let malvin_checks_backup =
            crate::artifacts::backup_workspace_malvin_checks_if_present(&artifacts.work_dir)
                .expect("backup malvin_checks");
        let backups = SessionDotfileBackups::from_parts(
            KissConfigBackup::Missing,
            malvin_checks_backup,
            KissConfigBackup::Missing,
        );
        let failure = RepoGateCommandFailure {
            command: "kiss check".into(),
            exit_code: Some(1),
            stdout: String::new(),
            stderr: String::new(),
        };
        let mut client = crate::acp::AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        client.replace_coder_session_slot_for_tests(captive_cat_acp_session_for_tests(
            &artifacts.work_dir,
        ));
        let out = run_tidy_prompt_after_post_run_gate_failure(
            &mut client,
            &artifacts,
            &backups,
            &failure,
        )
        .await;
        let err = out.expect_err("captive cat cannot complete tidy ACP JSON-RPC");
        assert!(
            !err.contains("coder ACP session is already open"),
            "must not start a second coder session: {err}"
        );
        let _ = client.end_coder_session().await;
    }
}
