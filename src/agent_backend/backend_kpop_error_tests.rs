//! Error-path `KPop` delegation tests for [`super::backend::AgentBackend`].

use super::backend_kpop_test_helpers::{
    empty_backups, mock_backend, mock_backend_bash_turn_exhaustion,
};
use super::{
    agent_backend_run_kpop_flow, agent_backend_run_kpop_multiturn,
};
use super::mini::MockStep;
use crate::acp::{AgentKpopMultiturnCtl, KpopFlowOnceArgs};
use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};

#[test]
fn agent_backend_run_kpop_flow_mini_stops_on_non_retryable_error() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut backend = mock_backend_bash_turn_exhaustion();
        let args = KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &["bad"],
            kpop_log: &tmp.path().join("kpop.log"),
        };
        let err = agent_backend_run_kpop_flow(&mut backend, &args, &empty_backups())
            .await
            .expect_err("non-retryable");
        assert!(err.0.contains("mini agent (kpop flow) failed"));
        assert!(err.0.contains("exhausted"));
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
        let mut backend = mock_backend_bash_turn_exhaustion();
        let builder = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
        let mut state =
            crate::kpop_progression::KpopMultiturnState::new(builder, exp_log, 2).expect("state");
        let backups = empty_backups();
        let ctl = AgentKpopMultiturnCtl {
            cwd: tmp.path(),
            kpop_log: tmp.path().join("kpop.log"),
            state: &mut state,
            session_dotfile_backups: &backups,
        };
        let err = agent_backend_run_kpop_multiturn(&mut backend, ctl)
            .await
            .expect_err("non-retryable");
        assert!(err.0.contains("mini agent (kpop multiturn) failed"));
        assert!(err.0.contains("exhausted"));
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
        let ctl = AgentKpopMultiturnCtl {
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
