use super::counters::{agent_declared_success, read_exp_log_text};

#[test]
fn kiss_cov_counter_wrapper_symbols() {
    let _ = (agent_declared_success, read_exp_log_text);
    let _ = stringify!(agent_declared_success);
    let _ = stringify!(read_exp_log_text);
}

#[test]
fn kiss_cov_counters_module_path_refs() {
    use crate::kpop_progression::counters::agent_declared_success;
    let text = "## Step 1 — KPop a\n";
    assert!(!agent_declared_success(text));
    assert!(agent_declared_success("## KPOP_SOLVED\n"));
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
