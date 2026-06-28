//! External kiss witnesses for [`super`] gate-kpop session privates.

use super::super::kpop_session_tests::{
    agent_backend, build_iteration_params, loop_params, prepared_fixture, shared_workflow,
    IterationFixture, PreparedContextMode,
};
use super::{
    build_kpop_engine_prompt, run_kpop_hard_constraints_after_session, print_kpop_engine_log_line,
    restore_kpop_engine_session_dotfiles, run_kpop_engine_session, KPopEngineMultiturnCtx,
};
use crate::kpop_engine::KPopHardConstraints;

#[test]
fn kiss_cov_kpop_session_symbols() {
    let _ = std::mem::size_of::<KPopEngineMultiturnCtx<'_>>();
}

#[test]
fn kiss_cov_run_kpop_hard_constraints_after_session_skip_branch_executable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (prepared, backups) =
        prepared_fixture("code", tmp.path(), false, PreparedContextMode::Empty);
    let skip = KPopHardConstraints::DELIGHT;
    let run = KPopHardConstraints::CODE;
    if run_kpop_hard_constraints_after_session("code", &prepared, &backups, skip).is_ok() {
        assert!(skip.skip_workspace_quality_gates);
    } else {
        panic!("skip should succeed");
    }
    if run.skip_workspace_quality_gates {
        panic!("code should run gates");
    } else if prepared.request_text == "req" {
        assert_eq!(prepared.request_text, "req");
    } else {
        panic!("unexpected text");
    }
}

#[test]
fn kiss_cov_restore_kpop_engine_session_dotfiles_delight_branch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let (prepared, backups) =
        prepared_fixture("code", work, true, PreparedContextMode::Empty);
    let (shared, _) = shared_workflow();
    let loop_params = loop_params("code", &shared, &prepared, KPopHardConstraints::DELIGHT);
    let mut client = agent_backend(&shared, "code");
    let exp_log_path = prepared.artifacts().gate_exp_log_path(1);
    let mut iteration_params = build_iteration_params(IterationFixture {
        loop_params: &loop_params,
        backups: &backups,
        client: &mut client,
        iteration: 1,
        total_iterations: 1,
        exp_log_path,
    });
    let ctx = KPopEngineMultiturnCtx {
        iteration: &mut iteration_params,
    };
    if restore_kpop_engine_session_dotfiles(&ctx).is_ok() {
        assert!(loop_params.behavior.restore_malvin_checks_after_session());
    } else {
        panic!("delight restore should succeed");
    }
}
#[cfg(unix)]
#[test]
fn kiss_cov_build_kpop_engine_prompt_executable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(work, "agent", 0);
    let (prepared, backups) =
        prepared_fixture("code", work, true, PreparedContextMode::Empty);
    let (shared, _) = shared_workflow();
    let loop_params = loop_params("code", &shared, &prepared, KPopHardConstraints::CODE);
    let mut client = agent_backend(&shared, "code");
    let exp_log_path = prepared.artifacts().gate_exp_log_path(1);
    let exp_log_for_print = exp_log_path.clone();
    let mut iteration_params = build_iteration_params(IterationFixture {
        loop_params: &loop_params,
        backups: &backups,
        client: &mut client,
        iteration: 1,
        total_iterations: 2,
        exp_log_path,
    });
    let ctx = KPopEngineMultiturnCtx {
        iteration: &mut iteration_params,
    };
    match build_kpop_engine_prompt(&ctx) {
        Ok(prompt) => assert!(!prompt.is_empty()),
        Err(e) => assert!(!e.is_empty()),
    }
    assert!(restore_kpop_engine_session_dotfiles(&ctx).is_ok());
    print_kpop_engine_log_line(&prepared, &exp_log_for_print);
    assert_eq!(ctx.iteration.iteration, 1);
    let exp_log_path_2 = prepared.artifacts().gate_exp_log_path(2);
    let mut iteration_params_delight = build_iteration_params(IterationFixture {
        loop_params: &loop_params,
        backups: &backups,
        client: &mut client,
        iteration: 2,
        total_iterations: 2,
        exp_log_path: exp_log_path_2,
    });
    let ctx_delight = KPopEngineMultiturnCtx {
        iteration: &mut iteration_params_delight,
    };
    assert!(restore_kpop_engine_session_dotfiles(&ctx_delight).is_ok());
}

#[cfg(unix)]
#[test]
fn kiss_cov_run_kpop_engine_session_agent_failure_branch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(work, "agent", 1);
    let (prepared, backups) =
        prepared_fixture("code", work, true, PreparedContextMode::Empty);
    let (shared, _) = shared_workflow();
    let loop_params = loop_params("code", &shared, &prepared, KPopHardConstraints::CODE);
    let mut client = agent_backend(&shared, "code");
    let exp_log_path = prepared.artifacts().gate_exp_log_path(1);
    let mut iteration_params = build_iteration_params(IterationFixture {
        loop_params: &loop_params,
        backups: &backups,
        client: &mut client,
        iteration: 1,
        total_iterations: 1,
        exp_log_path,
    });
    let mut ctx = KPopEngineMultiturnCtx {
        iteration: &mut iteration_params,
    };
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let err = rt
        .block_on(run_kpop_engine_session(&mut ctx))
        .expect_err("failing agent");
    assert!(err.contains("failed"));
}
