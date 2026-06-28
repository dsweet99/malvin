//! `KPop` delegation tests for [`super::backend::AgentBackend`].

use super::backend_kpop_test_helpers::{
    empty_backups, mini_done_backend, mock_backend,
};
use super::{
    agent_backend_attach_run_timing_for_session, agent_backend_run_kpop_flow,
    agent_backend_run_kpop_multiturn,
};
use super::mini::MockStep;
use crate::acp::{AgentKpopMultiturnCtl, KpopFlowOnceArgs};
use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};

#[test]
fn agent_backend_forwards_attach_run_timing_for_session() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let mut backend = mini_done_backend();
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
        let mut backend = mini_done_backend();
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
        let mut backend = mini_done_backend();
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
                MockStep::Ok(super::test_support::mini_done_response()),
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
fn agent_backend_run_kpop_multiturn_mini_succeeds_on_second_attempt() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let exp_log = tmp.path().join("exp.md");
        std::fs::write(&exp_log, "# exp\n").expect("exp log");
        let mut backend = mock_backend(
            vec![
                MockStep::RateLimited,
                MockStep::Ok(super::test_support::mini_done_response()),
            ],
            2,
        );
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
        agent_backend_run_kpop_multiturn(&mut backend, ctl)
            .await
            .expect("multiturn retry");
    });
}
