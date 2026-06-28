//! `KPop` flows on [`super::mini::MiniAgentClient`] (mini agent backend).

#[path = "kpop_bridge_prompt.rs"]
mod kpop_bridge_prompt;

use crate::acp::{
    kpop_fail_after_prompt, AgentError, AgentKpopMultiturnCtl, KpopFailAfterPrompt,
    KpopFlowOnceArgs,
};

use super::mini::MiniAgentClient;

use kpop_bridge_prompt::{guard_bridge_hypothesis_budget, restore_dotfiles_or_close, run_kpop_prompt};

pub(crate) async fn run_kpop_flow_once_mini(
    client: &mut MiniAgentClient,
    args: &KpopFlowOnceArgs<'_>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    client.begin_coder_session(args.cwd).await?;
    for prompt in args.kpop_prompts {
        if let Err(e) = run_kpop_prompt(client, prompt, args.kpop_log).await {
            client.end_coder_session().await.ok();
            return kpop_fail_after_prompt(KpopFailAfterPrompt {
                cwd: args.cwd,
                session_dotfile_backups,
                err: e,
                phase: "prompt",
            })
            .await;
        }
        restore_dotfiles_or_close(client, args.cwd, session_dotfile_backups).await?;
    }
    client.end_coder_session().await
}

pub(crate) async fn run_kpop_multiturn_once_mini(
    client: &mut MiniAgentClient,
    ctl: &mut AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    client.begin_coder_session(ctl.cwd).await?;

    loop {
        let prompt = match ctl.state.next_prompt() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(e) => {
                client.end_coder_session().await.ok();
                return Err(AgentError(e));
            }
        };
        if let Err(e) = run_kpop_prompt(client, prompt.as_str(), ctl.kpop_log.as_path()).await {
            client.end_coder_session().await.ok();
            return kpop_fail_after_prompt(KpopFailAfterPrompt {
                cwd: ctl.cwd,
                session_dotfile_backups: ctl.session_dotfile_backups,
                err: e,
                phase: "prompt",
            })
            .await;
        }
        restore_dotfiles_or_close(client, ctl.cwd, ctl.session_dotfile_backups).await?;
        guard_bridge_hypothesis_budget(client, ctl).await?;
        ctl.state.record_kpop_block_prompt_completed();
    }

    client.end_coder_session().await
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{run_kpop_flow_once_mini, run_kpop_multiturn_once_mini};
    use crate::acp::{AgentKpopMultiturnCtl, KpopFlowOnceArgs};
    use crate::agent_backend::mini::{LlmBackend, MiniAgentClient, MockScript, MockStep};
    use crate::agent_backend::test_support::{mini_done_response, mini_loop_config, test_io};
    use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
    use crate::kpop_progression::KpopMultiturnState;

    fn mock_client(responses: Vec<MockStep>) -> MiniAgentClient {
        MiniAgentClient::new_mock(
            mini_loop_config(4, 1),
            test_io(),
            LlmBackend::Mock(Mutex::new(MockScript {
                responses,
                call_count: 0,
                on_response: None,
            })),
        )
    }

    fn empty_backups() -> crate::artifacts::SessionDotfileBackups {
        crate::orchestrator::orchestrator_test_support::empty_dotfile_backups()
    }

    #[tokio::test]
    async fn run_kpop_flow_once_mini_completes_single_prompt() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = mock_client(vec![MockStep::Ok(mini_done_response())]);
        let log = tmp.path().join("kpop.log");
        let prompts = ["test prompt"];
        let args = KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &prompts,
            kpop_log: &log,
        };
        run_kpop_flow_once_mini(&mut client, &args, &empty_backups())
            .await
            .expect("flow once");
        assert!(!client.has_open_coder_session());
    }

    #[tokio::test]
    async fn run_kpop_flow_once_mini_fails_when_prompt_errors() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = mock_client(vec![MockStep::RateLimited]);
        let log = tmp.path().join("kpop.log");
        let prompts = ["fail prompt"];
        let args = KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &prompts,
            kpop_log: &log,
        };
        let err = run_kpop_flow_once_mini(&mut client, &args, &empty_backups())
            .await
            .expect_err("prompt failure");
        assert!(!err.0.is_empty());
        assert!(!client.has_open_coder_session());
    }

    #[tokio::test]
    async fn run_kpop_multiturn_once_mini_fails_when_hypothesis_budget_exceeded() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let exp_log = tmp.path().join("exp.md");
        std::fs::write(&exp_log, "# exp\n").expect("exp log");
        let exp_log_for_hook = exp_log.clone();
        let mut client = MiniAgentClient::new_mock(
            mini_loop_config(4, 1),
            test_io(),
            LlmBackend::Mock(Mutex::new(MockScript {
                responses: vec![MockStep::Ok(mini_done_response())],
                call_count: 0,
                on_response: Some(Box::new(move |_, _| {
                    std::fs::write(
                        &exp_log_for_hook,
                        "# exp\n## Step 1 — KPOP\n## Step 2 — KPOP\n## Step 3 — KPOP\n",
                    )
                    .expect("write exp");
                })),
            })),
        );
        let builder = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
        let mut state = KpopMultiturnState::new(builder, exp_log, 2).expect("state");
        let mut ctl = AgentKpopMultiturnCtl {
            cwd: tmp.path(),
            kpop_log: tmp.path().join("kpop.log"),
            state: &mut state,
            session_dotfile_backups: &empty_backups(),
        };
        let err = run_kpop_multiturn_once_mini(&mut client, &mut ctl)
            .await
            .expect_err("hypothesis budget");
        assert!(err.0.contains("hypothesis steps"));
    }

    fn kissconfig_dir_blocks_restore(work: &std::path::Path) {
        let kiss = work.join(".kissconfig");
        let _ = std::fs::remove_file(&kiss);
        std::fs::create_dir(&kiss).expect("kissconfig dir");
    }

    #[tokio::test]
    async fn run_kpop_flow_once_mini_closes_session_when_restore_fails() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".kissconfig"), b"k\n").expect("kissconfig");
        let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path())
            .expect("snapshot");
        let work = tmp.path().to_path_buf();
        let mut client = MiniAgentClient::new_mock(
            mini_loop_config(4, 1),
            test_io(),
            LlmBackend::Mock(Mutex::new(MockScript {
                responses: vec![MockStep::Ok(mini_done_response())],
                call_count: 0,
                on_response: Some(Box::new(move |_, _| {
                    kissconfig_dir_blocks_restore(&work);
                })),
            })),
        );
        let log = tmp.path().join("kpop.log");
        let prompts = ["test prompt"];
        let args = KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &prompts,
            kpop_log: &log,
        };
        let err = run_kpop_flow_once_mini(&mut client, &args, &backups)
            .await
            .expect_err("restore should fail");
        assert!(
            err.0.contains("restore"),
            "expected restore error, got: {}",
            err.0
        );
        assert!(
            !client.has_open_coder_session(),
            "mini session must be closed on restore failure"
        );
    }

    #[tokio::test]
    async fn run_kpop_multiturn_once_mini_preserves_single_message_history() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let exp_log = tmp.path().join("exp.md");
        std::fs::write(&exp_log, "# exp\n").expect("exp log");
        let mut client = mock_client(vec![MockStep::Ok(mini_done_response())]);
        let builder = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
        let mut state = KpopMultiturnState::new(builder, exp_log, 2).expect("state");
        let mut ctl = AgentKpopMultiturnCtl {
            cwd: tmp.path(),
            kpop_log: tmp.path().join("kpop.log"),
            state: &mut state,
            session_dotfile_backups: &empty_backups(),
        };
        run_kpop_multiturn_once_mini(&mut client, &mut ctl)
            .await
            .expect("multiturn once");
        assert!(!client.has_open_coder_session());
    }
}
