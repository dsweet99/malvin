use crate::kpop_engine::KPopHardConstraints;

use super::super::run_loop_exit::{gate_loop_early_exit, GateLoopExitCtx};
use super::run_loop_tests::gate_early_exit_fixture;
use super::{KPopEngineEarlyExitCtx, KpopEngineLoopIterationCtx};

#[test]
fn exhausted_loop_accepts_skipped_workspace_gates() {
    use super::super::kpop_session_tests::{
        loop_params, prepared_fixture, shared_workflow, PreparedContextMode,
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let (prepared, backups) =
        prepared_fixture("code", tmp.path(), false, PreparedContextMode::Empty);
    let (shared, _) = shared_workflow();
    let behavior = KPopHardConstraints::CODE.with_workspace_quality_gates(false);
    let params = loop_params("code", &shared, &prepared, behavior);
    assert!(super::exhausted_loop_gate_ok(&params, &backups));
}

#[test]
fn gate_loop_early_exit_requires_passing_gates() {
    let (_tmp, artifacts, backups, _bin, _guard) = gate_early_exit_fixture();
    let gate_ctx = GateLoopExitCtx {
        behavior: KPopHardConstraints::CODE,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
    };
    assert!(gate_loop_early_exit(&gate_ctx));
}

#[test]
fn kpop_engine_loop_ctx_types_are_constructible() {
    use std::sync::{Arc, Mutex};

    use super::super::kpop_session_tests::{agent_backend, loop_params, prepared_fixture, shared_workflow, PreparedContextMode};
    use crate::kpop_engine::KPopHardConstraints;

    let tmp = tempfile::tempdir().expect("tempdir");
    let (prepared, backups) = prepared_fixture("code", tmp.path(), false, PreparedContextMode::Empty);
    let (shared, _) = shared_workflow();
    let params = loop_params("code", &shared, &prepared, KPopHardConstraints::CODE);
    let run_timing = Arc::new(Mutex::new(crate::run_timing::RunTiming::default()));
    let mut client = agent_backend(&shared, "code");
    let _ctx = KpopEngineLoopIterationCtx {
        params: &params,
        iteration: 1,
        run_timing: &run_timing,
        client: &mut client,
    };
    let _ = KPopEngineEarlyExitCtx {
        behavior: KPopHardConstraints::CODE,
        consecutive_solved: 0,
        artifacts: &prepared.artifacts,
        session_dotfile_backups: &backups,
        agent_ran: true,
        run_timing: Some(&run_timing),
    };
}
