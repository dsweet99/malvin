use super::counters::{
    agent_declared_success, check_hypothesis_budget, count_kpop_entries, count_mbc2_entries,
    hypotheses_emitted, hypothesis_budget_exhausted, read_exp_log_text,
};

#[test]
fn kiss_cov_counter_wrapper_symbols() {
    let _ = (
        agent_declared_success,
        hypotheses_emitted,
        count_kpop_entries,
        count_mbc2_entries,
        read_exp_log_text,
    );
    let _ = stringify!(agent_declared_success);
    let _ = stringify!(hypotheses_emitted);
}
#[test]
fn kiss_cov_counters_module_path_refs() {
    use crate::kpop_progression::counters::{agent_declared_success, hypotheses_emitted};
    let text = "## Step 1 — KPop a\n";
    assert_eq!(hypotheses_emitted(text), 1);
    assert!(!agent_declared_success(text));
    assert!(agent_declared_success("## KPOP_SOLVED\n"));
}

#[test]
fn counts_steps_in_exp_log() {
    let text = "## Step 1 — KPop x\n## Step 2 — MBC2 y\n## Step 3 — KPop z\n";
    assert_eq!(count_kpop_entries(text), 2);
    assert_eq!(count_mbc2_entries(text), 1);
    assert_eq!(hypotheses_emitted(text), 3);
}

#[test]
fn agent_declared_success_detects_kpop_solved_marker() {
    assert!(!agent_declared_success(""));
    assert!(agent_declared_success("## KPOP_SOLVED\n"));
    assert!(agent_declared_success("## KPOP_SOLVED still going\n"));
    assert!(agent_declared_success("## KPOP_SOLVED — done\n"));
    assert!(!agent_declared_success("DONE\n"));
}

#[test]
fn read_exp_log_text_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "body\n").expect("write");
    assert_eq!(read_exp_log_text(&path).expect("read"), "body\n");
}

#[test]
fn check_hypothesis_budget_allows_at_limit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "## Step 1 — KPop a\n## Step 2 — KPop b\n").expect("write");
    check_hypothesis_budget(&path, 2).expect("at limit");
    assert!(check_hypothesis_budget(&path, 1).is_err());
}

#[test]
fn hypothesis_budget_exhausted_at_limit() {
    let text = "## Step 1 — KPop a\n## Step 2 — KPop b\n";
    assert!(hypothesis_budget_exhausted(text, 2));
    assert!(!hypothesis_budget_exhausted(text, 3));
}
