//! `KPop` delegation tests for [`super::backend::AgentBackend`].

use std::sync::Mutex;

use super::backend::AgentBackend;
use super::mini::{LlmBackend, MiniAgentClient, MiniLoopConfig, MockScript, MockStep};
use super::test_support::{mini_done_response, test_io};
use super::{
    agent_backend_attach_run_timing_for_session, agent_backend_run_kpop_flow,
    agent_backend_run_kpop_multiturn,
};
use crate::acp::KpopFlowOnceArgs;
use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use malvin_mini::CompletionResponse;

fn mock_backend(responses: Vec<MockStep>, max_http_retries: u32) -> AgentBackend {
    AgentBackend::Mini(MiniAgentClient::new_mock(
        MiniLoopConfig {
            model: "anthropic/claude-sonnet-4".into(),
            max_bash_turns: 4,
            max_http_retries,
        },
        test_io(),
        LlmBackend::Mock(Mutex::new(MockScript {
            responses,
            call_count: 0,
            on_response: None,
        })),
    ))
}

fn empty_backups() -> crate::artifacts::SessionDotfileBackups {
    crate::orchestrator::orchestrator_test_support::empty_dotfile_backups()
}

#[test]
fn agent_backend_forwards_attach_run_timing_for_session() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let mut backend = mock_backend(vec![MockStep::Ok(mini_done_response())], 1);
        let timing = agent_backend_attach_run_timing_for_session(&mut backend);
        assert!(std::sync::Arc::strong_count(&timing) >= 1);
        let tmp = tempfile::tempdir().expect("tempdir");
        backend.set_prompts_log_run_dir(Some(tmp.path().to_path_buf()));
        backend.begin_coder_session(tmp.path()).await.expect("begin");
        backend.end_coder_session().await.expect("end");
    });
}

#[test]
fn agent_backend_run_kpop_multiturn_mini_delegates() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let exp_log = tmp.path().join("exp.md");
        std::fs::write(&exp_log, "# exp\n").expect("exp log");
        let mut backend = mock_backend(vec![MockStep::Ok(mini_done_response())], 1);
        let builder = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
        let mut state =
            crate::kpop_progression::KpopMultiturnState::new(builder, exp_log, 2).expect("state");
        let backups = empty_backups();
        let ctl = crate::acp::AgentKpopMultiturnCtl {
            cwd: tmp.path(),
            kpop_log: tmp.path().join("kpop.log"),
            state: &mut state,
            session_dotfile_backups: &backups,
        };
        agent_backend_run_kpop_multiturn(&mut backend, ctl)
            .await
            .expect("multiturn");
    });
}

#[test]
fn agent_backend_run_kpop_flow_mini_delegates() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut backend = mock_backend(vec![MockStep::Ok(mini_done_response())], 1);
        let log = tmp.path().join("kpop.log");
        let prompts = ["hello"];
        let args = KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &prompts,
            kpop_log: &log,
        };
        agent_backend_run_kpop_flow(&mut backend, &args, &empty_backups())
            .await
            .expect("kpop flow");
    });
}

#[test]
fn agent_backend_run_kpop_flow_mini_succeeds_on_second_attempt() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut backend = mock_backend(
            vec![
                MockStep::RateLimited,
                MockStep::Ok(mini_done_response()),
            ],
            2,
        );
        let args = KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &["retry"],
            kpop_log: &tmp.path().join("kpop.log"),
        };
        agent_backend_run_kpop_flow(&mut backend, &args, &empty_backups())
            .await
            .expect("kpop flow retry");
    });
}

#[test]
fn agent_backend_run_kpop_flow_mini_stops_on_non_retryable_error() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut backend = mock_backend(
            vec![MockStep::Ok(CompletionResponse {
                content: "not a valid response without done".into(),
                usage: None,
            })],
            1,
        );
        let args = KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &["bad"],
            kpop_log: &tmp.path().join("kpop.log"),
        };
        let err = agent_backend_run_kpop_flow(&mut backend, &args, &empty_backups())
            .await
            .expect_err("non-retryable");
        assert!(err.0.contains("mini agent (kpop flow) failed"));
    });
}

#[test]
fn agent_backend_run_kpop_multiturn_mini_succeeds_on_second_attempt() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let exp_log = tmp.path().join("exp.md");
        std::fs::write(&exp_log, "# exp\n").expect("exp log");
        let mut backend = mock_backend(
            vec![
                MockStep::RateLimited,
                MockStep::Ok(mini_done_response()),
            ],
            2,
        );
        let builder = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
        let mut state =
            crate::kpop_progression::KpopMultiturnState::new(builder, exp_log, 2).expect("state");
        let backups = empty_backups();
        let ctl = crate::acp::AgentKpopMultiturnCtl {
            cwd: tmp.path(),
            kpop_log: tmp.path().join("kpop.log"),
            state: &mut state,
            session_dotfile_backups: &backups,
        };
        agent_backend_run_kpop_multiturn(&mut backend, ctl)
            .await
            .expect("multiturn retry");
    });
}

#[test]
fn agent_backend_run_kpop_flow_mini_reports_failure_after_retries() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut backend = mock_backend(vec![MockStep::RateLimited], 1);
        let args = KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &["fail"],
            kpop_log: &tmp.path().join("kpop.log"),
        };
        let err = agent_backend_run_kpop_flow(&mut backend, &args, &empty_backups())
            .await
            .expect_err("kpop flow should fail");
        assert!(err.0.contains("mini agent (kpop flow) failed"));
    });
}

#[test]
fn agent_backend_run_kpop_multiturn_mini_stops_on_non_retryable_error() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let exp_log = tmp.path().join("exp.md");
        std::fs::write(&exp_log, "# exp\n").expect("exp log");
        let mut backend = mock_backend(
            vec![MockStep::Ok(CompletionResponse {
                content: "not a valid response without done".into(),
                usage: None,
            })],
            1,
        );
        let builder = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
        let mut state =
            crate::kpop_progression::KpopMultiturnState::new(builder, exp_log, 2).expect("state");
        let backups = empty_backups();
        let ctl = crate::acp::AgentKpopMultiturnCtl {
            cwd: tmp.path(),
            kpop_log: tmp.path().join("kpop.log"),
            state: &mut state,
            session_dotfile_backups: &backups,
        };
        let err = agent_backend_run_kpop_multiturn(&mut backend, ctl)
            .await
            .expect_err("non-retryable");
        assert!(err.0.contains("mini agent (kpop multiturn) failed"));
    });
}

#[test]
fn agent_backend_run_kpop_multiturn_mini_reports_failure_after_retries() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let exp_log = tmp.path().join("exp.md");
        std::fs::write(&exp_log, "# exp\n").expect("exp log");
        let mut backend = mock_backend(vec![MockStep::RateLimited], 2);
        let builder = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
        let mut state =
            crate::kpop_progression::KpopMultiturnState::new(builder, exp_log, 2).expect("state");
        let backups = empty_backups();
        let ctl = crate::acp::AgentKpopMultiturnCtl {
            cwd: tmp.path(),
            kpop_log: tmp.path().join("kpop.log"),
            state: &mut state,
            session_dotfile_backups: &backups,
        };
        let err = agent_backend_run_kpop_multiturn(&mut backend, ctl)
            .await
            .expect_err("multiturn fail");
        assert!(err.0.contains("mini agent (kpop multiturn) failed"));
    });
}
