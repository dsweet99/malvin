use super::{ExperimentLog, StepHeadingKind};

#[test]
fn counts_steps_in_exp_log() {
    let log = ExperimentLog::from_text("## Step 1 — KPop x\n## Step 2 — MBC2 y\n## Step 3 — KPop z\n");
    assert_eq!(log.kpop_step_count(), 2);
    assert_eq!(log.mbc2_step_count(), 1);
    assert_eq!(log.hypothesis_step_count(), 3);
}

#[test]
fn declares_mpc_done_requires_exact_marker_line() {
    let log = ExperimentLog::from_text("## MPC_DONE extra\n");
    assert_eq!(log.mpc_done_marker_count(), 0);
    let log = ExperimentLog::from_text("## MPC_DONE\n");
    assert_eq!(log.mpc_done_marker_count(), 1);
    assert_eq!(
        ExperimentLog::from_text("## MPC_DONE\n## MPC_DONE\n").mpc_done_marker_count(),
        2
    );
    assert_eq!(ExperimentLog::from_text("preamble\n").mpc_done_marker_count(), 0);
    assert_eq!(
        ExperimentLog::from_text("  ## MPC_DONE\n").mpc_done_marker_count(),
        1
    );
    assert_eq!(
        ExperimentLog::from_text("## MPC_DONE   \n").mpc_done_marker_count(),
        1
    );
    assert_eq!(
        ExperimentLog::from_text("## MPC_DONE trailing\n").mpc_done_marker_count(),
        0
    );
    assert_eq!(
        ExperimentLog::from_text("## MPC_DONE-ish\n").mpc_done_marker_count(),
        0
    );
}

#[test]
fn declares_kpop_solved_requires_exact_marker_line() {
    let log = ExperimentLog::from_text("## KPOP_SOLVED extra\n");
    assert!(!log.declares_kpop_solved());
    let log = ExperimentLog::from_text("## KPOP_SOLVED\n");
    assert!(log.declares_kpop_solved());
    assert_eq!(
        ExperimentLog::from_text("## KPOP_SOLVED\n## KPOP_SOLVED\n").kpop_solved_marker_count(),
        2
    );
    assert_eq!(ExperimentLog::from_text("preamble\n").kpop_solved_marker_count(), 0);
    assert_eq!(
        ExperimentLog::from_text("  ## KPOP_SOLVED\n").kpop_solved_marker_count(),
        1
    );
    assert_eq!(
        ExperimentLog::from_text("## KPOP_SOLVED   \n").kpop_solved_marker_count(),
        1
    );
    assert_eq!(
        ExperimentLog::from_text("## KPOP_SOLVED trailing\n").kpop_solved_marker_count(),
        0
    );
    assert_eq!(
        ExperimentLog::from_text("## KPOP_SOLVED-ish\n").kpop_solved_marker_count(),
        0
    );
}

#[test]
fn read_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "body\n").expect("write");
    assert_eq!(ExperimentLog::read(&path).expect("read").as_str(), "body\n");
}

#[test]
fn check_hypothesis_budget_ok_and_over() {
    let log = ExperimentLog::from_text("## Step 1 — KPop x\n");
    assert!(log.check_hypothesis_budget(1).is_ok());
    let err = log.check_hypothesis_budget(0).expect_err("over budget");
    assert!(err.contains("exceeding --max-hypotheses"));
}

#[test]
fn step_kind_classifies_kpop_mbc2_and_rejects_kpopulation() {
    use super::step_kind;
    assert_eq!(step_kind("## Step 1 — KPop x"), Some(StepHeadingKind::KPop));
    assert_eq!(step_kind("## Step 2 — MBC2 y"), Some(StepHeadingKind::Mbc2));
    assert_eq!(step_kind("## Step 3 — kpopulation x"), None);
}
