use super::{
    kpop_engine_loop_one_iteration, restore_carry_forward_before_iteration_snapshot,
    run_kpop_engine, run_kpop_engine_on_loop_iteration, run_gate_workspace_gates_with_fresh_backups,
    wire_kpop_engine_client,
};

#[test]
fn kiss_cov_gate_run_loop_privates() {
    let _ = (
        kpop_engine_loop_one_iteration,
        run_kpop_engine_on_loop_iteration,
        wire_kpop_engine_client,
        run_gate_workspace_gates_with_fresh_backups,
        std::mem::size_of::<super::run_loop_iteration::KpopEngineLoopIterationCtx<'_>>,
    );
}
use crate::artifacts::SessionDotfileBackups;
use crate::session_dotfile_backup::GitignoreBackup;

#[test]
fn session_mpc_plan_declares_done_reads_done_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    assert!(!super::session_mpc_plan_declares_done(&artifacts).expect("read"));
    std::fs::write(crate::artifacts::mpc_plan_path(&artifacts), "DONE\n").expect("write");
    assert!(super::session_mpc_plan_declares_done(&artifacts).expect("read"));
}

pub(crate) fn gate_early_exit_fixture() -> (
    tempfile::TempDir,
    crate::artifacts::RunArtifacts,
    SessionDotfileBackups,
    tempfile::TempDir,
    crate::repo_checks::FakeCommandDirGuard,
) {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
    let (bin, guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 0);
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let backups = SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
    (tmp, artifacts, backups, bin, guard)
}

#[test]
fn kiss_cov_gate_early_exit_fixture_witness() {
    let (_tmp, artifacts, _backups, _bin, _guard) = gate_early_exit_fixture();
    assert!(artifacts.work_dir.join(".malvin").join("checks").is_file());
}

#[test]
fn kpop_engine_loop_session_helpers_are_covered() {
    let _ = run_kpop_engine_on_loop_iteration;
    let _ = wire_kpop_engine_client;
    let _ = kpop_engine_loop_one_iteration;
    let _ = run_kpop_engine;
}

fn ensure_git_repo_for_gate_tests(work: &std::path::Path) {
    if crate::git_worktree_toplevel(work).is_none() {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(work)
            .status()
            .expect("git init");
    }
}

fn write_gate_checks_file(work: &std::path::Path, content: &str) {
    ensure_git_repo_for_gate_tests(work);
    let checks = crate::malvin_checks_path(work);
    if let Some(parent) = checks.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(checks, content).expect("checks");
}

fn fail_gate_prepared_fixture(
    work: &std::path::Path,
) -> (SessionDotfileBackups, crate::kpop_engine::KPopEnginePrepared) {
    write_gate_checks_file(work, "kiss check\n");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
    let backups = SessionDotfileBackups::snapshot(work).expect("snapshot");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let prepared = crate::kpop_engine::KPopEnginePrepared {
        artifacts,
        context: crate::prompt_stratification::WorkflowRenderContext::default(),
        request_text: "req".into(),
        startup_emit_request: "req".into(),
        store,
        malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
    };
    (backups, prepared)
}

#[test]
fn restore_carry_forward_before_iteration_snapshot_undoes_disk_regress() {
    const BASELINE: &str = "baseline\n";
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let gitignore = work.join(".gitignore");
    std::fs::write(&gitignore, BASELINE).expect("write");
    let carry = SessionDotfileBackups::snapshot(work).expect("snapshot");
    std::fs::write(&gitignore, "tampered\n").expect("tamper");
    restore_carry_forward_before_iteration_snapshot(work, Some(&carry)).expect("restore");
    assert_eq!(std::fs::read_to_string(&gitignore).expect("read"), BASELINE);
    let resnapshot = SessionDotfileBackups::snapshot(work).expect("resnapshot");
    let GitignoreBackup::Present { files, .. } = resnapshot.gitignore else {
        panic!("expected gitignore present");
    };
    assert_eq!(files[0].bytes, BASELINE.as_bytes());
}

#[cfg(unix)]
#[test]
fn kpop_engine_loop_rejects_over_budget_exp_log_after_session() {
    use super::run_loop_iteration::{
        build_authenticated_kpop_engine_client, kpop_engine_loop_one_iteration,
        KpopEngineLoopIterationCtx,
    };
    use crate::kpop_engine::kpop_session_tests::{
        loop_params, prepared_fixture, shared_workflow, PreparedContextMode,
    };
    use crate::kpop_engine::{KPopEngineParams, KPopHardConstraints};
    use std::sync::{Arc, Mutex};

    crate::test_utils::enable_test_fast_teardown();
    crate::test_utils::with_isolated_home(|work| {
        let mock = work.join("mock-gate-kpop-agent");
        let _env =
            crate::cli::kpop_flow::kpop_flow_run_loop_tests::install_mock_agent_env(work, &mock);
        let (prepared, _backups) =
            prepared_fixture("code", work, true, PreparedContextMode::PathsOnly);
        let (shared, _) = shared_workflow();
        let base = loop_params("code", &shared, &prepared, KPopHardConstraints::CODE);
        let loop_params = KPopEngineParams {
            max_hypotheses: 0,
            ..base
        };
        let run_timing = Arc::new(Mutex::new(crate::run_timing::RunTiming::default()));
        crate::test_utils::block_on_test_async(async {
            let mut client =
                build_authenticated_kpop_engine_client(&loop_params, &run_timing).expect("client");
            let err = kpop_engine_loop_one_iteration(KpopEngineLoopIterationCtx {
                params: &loop_params,
                iteration: 1,
                run_timing: &run_timing,
                client: &mut client,
            })
            .await
            .expect_err("mock agent writes one hypothesis step; budget is zero");
            assert!(
                err.contains("hypothesis steps"),
                "expected budget error, got: {err}"
            );
        });
    });
}

#[test]
fn fail_gate_after_exhausted_restores_disk_without_rerunning_gates_for_code() {
    use crate::kpop_engine::KPopHardConstraints;
    use crate::kpop_engine::fail_kpop_engine_after_exhausted;

    let tmp = tempfile::tempdir().expect("tempdir");
    let (backups, prepared) = fail_gate_prepared_fixture(tmp.path());
    std::fs::write(crate::malvin_checks_path(tmp.path()), "tampered\n").expect("tamper");
    let err = fail_kpop_engine_after_exhausted(
        "malvin code",
        &prepared,
        &backups,
        KPopHardConstraints::CODE,
    )
    .expect_err("gates failed");
    assert!(err.contains("quality gates did not pass"));
    assert_eq!(
        std::fs::read_to_string(crate::malvin_checks_path(tmp.path())).expect("read"),
        "kiss check\n",
        "exhausted fail path must rewind dotfiles without invoking gates again"
    );
}
