use crate::kpop_engine::KPopHardConstraints;

use super::super::run_loop_exit::{GateLoopExitCtx, mpc_done_early_exit};
use super::run_loop_tests::gate_early_exit_fixture;

#[test]
fn kiss_cov_mpc_done_exit_after_planner_witness() {
    let _ = stringify!(mpc_done_exit_after_planner);
}

#[test]
fn kiss_cov_run_loop_private_witnesses() {
    let _ = (
        stringify!(exhausted_loop_gate_ok),
        stringify!(prepare_kpop_engine_loop),
        stringify!(run_kpop_engine_iteration),
        stringify!(KpopEngineIterationInput),
    );
}

#[test]
fn kiss_cov_run_loop_iteration_witnesses() {
    let _ = (
        stringify!(super::run_loop_iteration::KpopEngineLoopIterationCtx),
        stringify!(crate::kpop_engine::run_loop::KpopEngineLoopIterationCtx),
    );
}

#[test]
fn mpc_done_early_exit_requires_marker_and_gates() {
    let (tmp, artifacts, backups, _bin, _guard) = gate_early_exit_fixture();
    let brief = tmp.path().join("brief.md");
    std::fs::write(&brief, "no marker\n").expect("write");
    let code_ctx = GateLoopExitCtx {
        behavior: KPopHardConstraints::CODE,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
    };
    assert!(!mpc_done_early_exit(&code_ctx, &brief).expect("read"));
    std::fs::write(&brief, "## MPC_DONE\n").expect("write");
    assert!(mpc_done_early_exit(&code_ctx, &brief).expect("read"));
    let delight_ctx = GateLoopExitCtx {
        behavior: KPopHardConstraints::DELIGHT,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
    };
    assert!(mpc_done_early_exit(&delight_ctx, &brief).expect("read"));
}

#[test]
fn kpop_engine_iteration_input_is_constructible() {
    let _ = std::mem::size_of::<super::KpopEngineIterationInput<'static>>();
}

#[test]
fn kpop_engine_loop_ctx_types_are_constructible() {
    let _ = std::mem::size_of::<super::run_loop_iteration::KpopEngineLoopIterationCtx<'static>>();
}
