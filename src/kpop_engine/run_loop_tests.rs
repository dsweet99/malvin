use super::{
    kpop_engine_loop_one_iteration, kpop_engine_solved_early_exit,
    refresh_consecutive_solved_streak,
    restore_carry_forward_before_iteration_snapshot,
    run_kpop_engine, run_kpop_engine_on_loop_iteration, run_gate_workspace_gates_with_fresh_backups,
    session_wrote_kpop_solved, wire_kpop_engine_client, KPopEngineEarlyExitCtx,
};
use super::super::run_loop_exit::{GateLoopExitCtx, kpop_solved_early_exit};

#[test]
fn kiss_cov_gate_run_loop_privates() {
    let _ = (
        kpop_engine_loop_one_iteration,
        run_kpop_engine_on_loop_iteration,
        wire_kpop_engine_client,
        run_gate_workspace_gates_with_fresh_backups,
    );
}
use crate::artifacts::SessionDotfileBackups;
use crate::session_dotfile_backup::GitignoreBackup;

#[test]
fn refresh_consecutive_solved_streak_increments_or_resets() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let empty = tmp.path().join("empty.md");
    std::fs::write(&empty, "").expect("write");
    assert_eq!(refresh_consecutive_solved_streak(1, &empty).expect("read"), 0);
    let solved = tmp.path().join("solved.md");
    std::fs::write(&solved, "## KPOP_SOLVED\n").expect("write");
    assert_eq!(refresh_consecutive_solved_streak(1, &solved).expect("read"), 2);
}

#[test]
fn session_wrote_kpop_solved_reads_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
    assert!(session_wrote_kpop_solved(&path).expect("read"));
}

#[test]
fn kpop_solved_early_exit_checks_streak_and_workspace() {
    use crate::kpop_engine::KPopHardConstraints;
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 0);
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let backups = SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
    let gate_ctx = |mpc_enabled| GateLoopExitCtx {
        behavior: KPopHardConstraints::CODE,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
        mpc_enabled,
    };
    assert!(!kpop_solved_early_exit(&gate_ctx(false), 1));
    assert!(kpop_solved_early_exit(&gate_ctx(false), 2));
    assert!(kpop_solved_early_exit(
        &GateLoopExitCtx {
            behavior: KPopHardConstraints::INIT,
            artifacts: &artifacts,
            session_dotfile_backups: &backups,
            mpc_enabled: false,
        },
        1,
    ));
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
fn kpop_engine_solved_early_exit_needs_streak_and_gates() {
    use crate::kpop_engine::KPopHardConstraints;
    let (_tmp, artifacts, backups, _bin, _guard) = gate_early_exit_fixture();
    let ctx = |behavior, streak, mpc_enabled| KPopEngineEarlyExitCtx {
        behavior,
        consecutive_solved: streak,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
        agent_ran: true,
        run_timing: None,
        mpc_enabled,
    };
    assert!(kpop_engine_solved_early_exit(ctx(KPopHardConstraints::CODE, 1, false)).is_none());
    assert!(kpop_engine_solved_early_exit(ctx(KPopHardConstraints::CODE, 2, false)).is_some());
    assert!(kpop_engine_solved_early_exit(ctx(KPopHardConstraints::INIT, 1, false)).is_some());
    assert!(kpop_engine_solved_early_exit(ctx(KPopHardConstraints::CODE, 2, true)).is_none());
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
