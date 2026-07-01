use super::counters::{
    agent_declared_success, count_kpop_entries, count_kpop_solved_markers, count_mbc2_entries,
    hypotheses_emitted, read_exp_log_text,
};

#[test]
fn kiss_cov_counter_wrapper_symbols() {
    let _ = (
        agent_declared_success,
        hypotheses_emitted,
        count_kpop_entries,
        count_mbc2_entries,
        count_kpop_solved_markers,
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
}

#[test]
fn counts_steps_in_exp_log() {
    let text = "## Step 1 — KPop x\n## Step 2 — MBC2 y\n## Step 3 — KPop z\n";
    assert_eq!(count_kpop_entries(text), 2);
    assert_eq!(count_mbc2_entries(text), 1);
    assert_eq!(hypotheses_emitted(text), 3);
}

#[test]
fn agent_declared_success_requires_exact_marker_line() {
    assert!(!agent_declared_success("## KPOP_SOLVED extra\n"));
    assert!(agent_declared_success("## KPOP_SOLVED\n"));
    assert_eq!(count_kpop_solved_markers("## KPOP_SOLVED\n## KPOP_SOLVED\n"), 2);
    assert_eq!(count_kpop_solved_markers("preamble\n"), 0);
    assert_eq!(count_kpop_solved_markers("  ## KPOP_SOLVED\n"), 1);
    assert_eq!(count_kpop_solved_markers("## KPOP_SOLVED   \n"), 1);
    assert_eq!(count_kpop_solved_markers("## KPOP_SOLVED trailing\n"), 0);
    assert_eq!(count_kpop_solved_markers("## KPOP_SOLVED-ish\n"), 0);
}

#[test]
fn read_exp_log_text_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "body\n").expect("write");
    assert_eq!(read_exp_log_text(&path).expect("read"), "body\n");
}
