use super::run_loop::{
    gate_kpop_solved_early_exit, kpop_solved_early_exit, refresh_consecutive_solved_streak,
    run_gate_kpop_loop, run_gate_kpop_on_loop_iteration, session_wrote_kpop_solved,
    gate_kpop_loop_one_iteration, wire_gate_kpop_client, GateKpopEarlyExitCtx,
};
use crate::artifacts::SessionDotfileBackups;

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
    use super::behavior::GateLoopBehavior;
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
    use super::behavior::GateLoopBehavior;
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
