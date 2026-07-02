use super::{ExperimentLog, StepHeadingKind};

#[test]
fn counts_steps_in_exp_log() {
    let log = ExperimentLog::from_text("## Step 1 — KPop x\n## Step 2 — MBC2 y\n## Step 3 — KPop z\n");
    assert_eq!(log.kpop_step_count(), 2);
    assert_eq!(log.mbc2_step_count(), 1);
    assert_eq!(log.hypothesis_step_count(), 3);
}

#[test]
fn read_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "body\n").expect("write");
    assert_eq!(ExperimentLog::read(&path).expect("read").as_str(), "body\n");
}
#[test]
fn step_kind_classifies_kpop_mbc2_and_rejects_kpopulation() {
    use super::step_kind;
    assert_eq!(step_kind("## Step 1 — KPop x"), Some(StepHeadingKind::KPop));
    assert_eq!(step_kind("## Step 2 — MBC2 y"), Some(StepHeadingKind::Mbc2));
    assert_eq!(step_kind("## Step 3 — kpopulation x"), None);
}
