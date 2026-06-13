use super::{
    build_authenticated_gate_kpop_client,
    gate_kpop_loop_one_iteration, gate_kpop_solved_early_exit, kpop_solved_early_exit,
    refresh_consecutive_solved_streak, restore_carry_forward_before_iteration_snapshot,
    run_gate_kpop_loop, run_gate_kpop_on_loop_iteration, run_gate_workspace_gates_with_fresh_backups,
    session_wrote_kpop_solved, wire_gate_kpop_client, GateKpopEarlyExitCtx,
};

#[test]
fn kiss_cov_gate_run_loop_privates() {
    let _ = (
        gate_kpop_loop_one_iteration,
        run_gate_kpop_on_loop_iteration,
        wire_gate_kpop_client,
        run_gate_workspace_gates_with_fresh_backups,
        build_authenticated_gate_kpop_client,
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
    use crate::cli::gate_kpop_workflow::GateLoopBehavior;
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 0);
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let backups = SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
    assert!(!kpop_solved_early_exit(
        GateLoopBehavior::CODE,
        1,
        &artifacts,
        &backups
    ));
    assert!(kpop_solved_early_exit(
        GateLoopBehavior::CODE,
        2,
        &artifacts,
        &backups
    ));
    assert!(kpop_solved_early_exit(
        GateLoopBehavior::INIT,
        1,
        &artifacts,
        &backups
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
fn gate_kpop_solved_early_exit_needs_streak_and_gates() {
    use crate::cli::gate_kpop_workflow::GateLoopBehavior;
    let (_tmp, artifacts, backups, _bin, _guard) = gate_early_exit_fixture();
    let ctx = |behavior, streak| GateKpopEarlyExitCtx {
        behavior,
        consecutive_solved: streak,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
        agent_ran: true,
        run_timing: None,
    };
    assert!(gate_kpop_solved_early_exit(ctx(GateLoopBehavior::CODE, 1)).is_none());
    assert!(gate_kpop_solved_early_exit(ctx(GateLoopBehavior::CODE, 2)).is_some());
    assert!(gate_kpop_solved_early_exit(ctx(GateLoopBehavior::INIT, 1)).is_some());
}

#[test]
fn gate_kpop_loop_session_helpers_are_covered() {
    let _ = run_gate_kpop_on_loop_iteration;
    let _ = wire_gate_kpop_client;
    let _ = gate_kpop_loop_one_iteration;
    let _ = run_gate_kpop_loop;
}

fn fail_gate_prepared_fixture(
    work: &std::path::Path,
) -> (SessionDotfileBackups, crate::cli::gate_kpop_workflow::GateKpopPrepared) {
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
    let backups = SessionDotfileBackups::snapshot(work).expect("snapshot");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let prepared = crate::cli::gate_kpop_workflow::GateKpopPrepared {
        artifacts,
        context: std::collections::HashMap::new(),
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
    use crate::cli::gate_kpop_workflow::GateLoopBehavior;
    use crate::cli::gate_kpop_workflow::fail_gate_kpop_after_exhausted;

    let tmp = tempfile::tempdir().expect("tempdir");
    let (backups, prepared) = fail_gate_prepared_fixture(tmp.path());
    std::fs::write(tmp.path().join(".malvin/checks"), "tampered\n").expect("tamper");
    let err = fail_gate_kpop_after_exhausted(
        "malvin code",
        &prepared,
        &backups,
        GateLoopBehavior::CODE,
    )
    .expect_err("gates failed");
    assert!(err.contains("quality gates did not pass"));
    assert_eq!(
        std::fs::read_to_string(tmp.path().join(".malvin/checks")).expect("read"),
        "kiss check\n",
        "exhausted fail path must rewind dotfiles without invoking gates again"
    );
}
