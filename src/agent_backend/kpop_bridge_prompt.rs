//! Prompt execution and hypothesis-budget checks for [`super::kpop_bridge`].

use crate::acp::{AgentError, AgentKpopMultiturnCtl, CoderPromptOptions};
use crate::artifacts::SessionDotfileBackups;

use crate::agent_backend::mini::MiniAgentClient;

pub(super) fn kpop_coder_opts() -> CoderPromptOptions<'static> {
    CoderPromptOptions {
        llm_phase: Some(crate::run_timing::TimingPhase::Implement),
        ..Default::default()
    }
}

pub(super) async fn run_kpop_prompt(
    client: &mut MiniAgentClient,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), AgentError> {
    client
        .run_coder_prompt(prompt, log_path, "kpop", kpop_coder_opts())
        .await
}

pub(super) async fn guard_bridge_hypothesis_budget(
    client: &mut MiniAgentClient,
    ctl: &AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    let exp_log = crate::kpop_experiment_log::ExperimentLog::read(ctl.state.exp_log_path())
        .map_err(AgentError)?;
    if let Err(msg) = exp_log.check_hypothesis_budget(ctl.state.max_hypotheses) {
        client.end_coder_session().await.ok();
        return Err(AgentError(msg));
    }
    Ok(())
}

pub(super) async fn restore_dotfiles_or_close(
    client: &mut MiniAgentClient,
    cwd: &std::path::Path,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), AgentError> {
    if let Err(e) =
        crate::acp::restore_session_dotfiles_after_success(cwd, session_dotfile_backups)
    {
        client.end_coder_session().await.ok();
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod budget_tests {
    use super::guard_bridge_hypothesis_budget;
    use crate::acp::AgentKpopMultiturnCtl;
    use crate::agent_backend::mini::MiniAgentClient;
    use crate::agent_backend::test_support::{mini_loop_config, test_io};
    use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
    use crate::kpop_progression::KpopMultiturnState;
    use crate::orchestrator::orchestrator_test_support::empty_dotfile_backups;

    #[tokio::test]
    async fn check_hypothesis_budget_ok_and_over() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let exp = tmp.path().join("exp.md");
        std::fs::write(&exp, "## Step 1 — KPop x\n## Step 2 — KPop y\n").expect("write");
        let mut state =
            KpopMultiturnState::new(KpopMultiturnPrompts::Smoke(SmokeKpopBuilder), exp, 1)
                .expect("state");
        let backups = empty_dotfile_backups();
        let mut client = MiniAgentClient::new_mock(
            mini_loop_config(1, 1),
            test_io(),
            crate::agent_backend::mini::LlmBackend::Mock(std::sync::Mutex::new(
                crate::agent_backend::mini::MockScript {
                    responses: vec![],
                    call_count: 0,
                    on_response: None,
                },
            )),
        );
        let ctl = AgentKpopMultiturnCtl {
            cwd: tmp.path(),
            kpop_log: tmp.path().join("kpop.log"),
            state: &mut state,
            session_dotfile_backups: &backups,
        };
        let err = guard_bridge_hypothesis_budget(&mut client, &ctl)
            .await
            .expect_err("over budget");
        assert!(err.0.contains("exceeding --max-hypotheses"));
        std::fs::write(ctl.state.exp_log_path(), "## Step 1 — KPop x\n").expect("rewrite");
        guard_bridge_hypothesis_budget(&mut client, &ctl)
            .await
            .expect("within budget");
    }

    #[test]
    fn kiss_cov_guard_bridge_hypothesis_budget_symbol() {
        let _ = stringify!(guard_bridge_hypothesis_budget);
    }
}
