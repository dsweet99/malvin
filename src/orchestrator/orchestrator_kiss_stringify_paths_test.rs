//! Regression guards for `orchestrator_kiss_stringify.rs` kiss path strings.

const SRC: &str = include_str!("orchestrator_kiss_stringify.rs");

const TOKIO_COVERED_SYMBOLS: &[&str] = &["run_bug_remediation_gap", "run_check_plan"];

const FN_ITEM_COVERED_SYMBOLS: &[&str] = &[
    "run_concerns_and_check_abort_impl",
    "run_code_review_phase",
    "run_reviewers_spawn_coder_session",
    "run_review_write_coder_session",
    "ensure_review_prep_after_reviewers_spawn",
    "ensure_artifact_review_after_review_write",
    "artifact_review_lgtm_after_review_write",
];

fn line_stringifies_exact_symbol(line: &str, sym: &str) -> bool {
    line.contains("stringify!") && line.contains(&format!("::{sym})"))
}

#[test]
fn orchestrator_kiss_must_not_stringify_tokio_smoke_symbols() {
    for line in SRC.lines() {
        for sym in TOKIO_COVERED_SYMBOLS {
            assert!(
                !line_stringifies_exact_symbol(line, sym),
                "orchestrator_kiss_stringify must not stringify {sym}; tokio smokes in orchestrator_kiss_coverage.rs cover it"
            );
        }
    }
}

#[test]
fn orchestrator_kiss_must_not_stringify_fn_item_symbols() {
    for line in SRC.lines() {
        for sym in FN_ITEM_COVERED_SYMBOLS {
            assert!(
                !line_stringifies_exact_symbol(line, sym),
                "orchestrator_kiss_stringify must not stringify {sym}; fn-item refs in orchestrator_kiss_coverage.rs cover it"
            );
        }
    }
}

#[test]
fn orchestrator_kiss_must_reference_check_plan_review_file() {
    assert!(
        SRC.contains("check_plan::read_check_plan_review_file"),
        "orchestrator_kiss_stringify must reference check_plan::read_check_plan_review_file"
    );
}
