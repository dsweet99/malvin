//! `KPop` flows on [`super::mini::MiniAgentClient`] (mini agent backend).

use crate::acp::{
    kpop_fail_after_prompt, restore_session_dotfiles_after_success,
    AgentError, AgentKpopMultiturnCtl, CoderPromptOptions, KpopFailAfterPrompt, KpopFlowOnceArgs,
};

use super::mini::MiniAgentClient;

fn kpop_coder_opts() -> CoderPromptOptions<'static> {
    CoderPromptOptions {
        llm_phase: Some(crate::run_timing::TimingPhase::Implement),
        ..Default::default()
    }
}

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
        restore_session_dotfiles_after_success(args.cwd, session_dotfile_backups)?;
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
        restore_session_dotfiles_after_success(ctl.cwd, ctl.session_dotfile_backups)?;
        check_hypothesis_budget(client, ctl).await?;
        ctl.state.record_kpop_block_prompt_completed();
    }

    client.end_coder_session().await
}

async fn run_kpop_prompt(
    client: &mut MiniAgentClient,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), AgentError> {
    client
        .run_coder_prompt(prompt, log_path, "kpop", kpop_coder_opts())
        .await
}

async fn check_hypothesis_budget(
    client: &mut MiniAgentClient,
    ctl: &AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    let exp_text = crate::kpop_progression::read_exp_log_text(ctl.state.exp_log_path())
        .map_err(AgentError)?;
    let hypotheses_after = crate::kpop_progression::hypotheses_emitted(&exp_text);
    if hypotheses_after > ctl.state.max_hypotheses {
        client.end_coder_session().await.ok();
        return Err(AgentError(format!(
            "experiment log counts {hypotheses_after} hypothesis steps, exceeding --max-hypotheses ({})",
            ctl.state.max_hypotheses
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{run_kpop_flow_once_mini, run_kpop_multiturn_once_mini};
    use crate::acp::{AgentKpopMultiturnCtl, KpopFlowOnceArgs};
    use crate::agent_backend::mini::{LlmBackend, MiniAgentClient, MiniLoopConfig, MockScript, MockStep};
    use crate::agent_backend::test_support::{mini_done_response, test_io};
    use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
    use crate::kpop_progression::KpopMultiturnState;

    fn mock_client(responses: Vec<MockStep>) -> MiniAgentClient {
        MiniAgentClient::new_mock(
            MiniLoopConfig {
                model: "anthropic/claude-sonnet-4".into(),
                max_bash_turns: 4,
                max_http_retries: 1,
            },
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
            MiniLoopConfig {
                model: "anthropic/claude-sonnet-4".into(),
                max_bash_turns: 4,
                max_http_retries: 1,
            },
            test_io(),
            LlmBackend::Mock(Mutex::new(MockScript {
                responses: vec![MockStep::Ok(mini_done_response())],
                call_count: 0,
                on_response: Some(Box::new(move |_| {
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
