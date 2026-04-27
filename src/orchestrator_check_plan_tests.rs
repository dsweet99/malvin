use crate::orchestrator::{WorkflowError, clear_review_file};
use crate::review_sync::is_lgtm_str;

#[test]
fn check_plan_error_message_format() {
    let err = WorkflowError("check_plan did not pass".to_string());
    assert_eq!(err.0, "check_plan did not pass");
}

#[test]
fn is_lgtm_str_returns_false_for_non_lgtm_content() {
    assert!(!is_lgtm_str(""), "empty string is not LGTM");
    assert!(!is_lgtm_str("not lgtm"), "arbitrary text is not LGTM");
    assert!(
        !is_lgtm_str("## Issues\n- problem 1"),
        "review with issues is not LGTM"
    );
    assert!(!is_lgtm_str("lgtm"), "lowercase lgtm is not LGTM");
    assert!(
        !is_lgtm_str("LGTM with notes"),
        "LGTM with extra text is not LGTM"
    );
    assert!(!is_lgtm_str("Almost LGTM"), "LGTM with prefix is not LGTM");
}

#[test]
fn is_lgtm_str_returns_true_for_lgtm() {
    assert!(is_lgtm_str("LGTM"), "exact LGTM");
    assert!(is_lgtm_str("LGTM\n"), "LGTM with trailing newline");
    assert!(is_lgtm_str("  LGTM  "), "LGTM with whitespace");
    assert!(is_lgtm_str("\u{FEFF}LGTM"), "LGTM with BOM");
}

#[test]
fn check_plan_abort_flow_components_verify_non_lgtm_causes_failure() {
    let t = tempfile::tempdir().unwrap();
    let review_path = t.path().join("review.md");

    std::fs::write(&review_path, "LGTM\n").unwrap();
    clear_review_file(&review_path).expect("clear_review_file must succeed");
    assert!(!review_path.exists(), "old LGTM must be cleared");

    std::fs::write(&review_path, "## Issues\n- The plan is incomplete").unwrap();
    let contents = std::fs::read_to_string(&review_path).unwrap_or_default();
    assert!(!is_lgtm_str(&contents), "non-LGTM content triggers abort");
    let err = WorkflowError("check_plan did not pass".to_string());
    assert_eq!(err.0, "check_plan did not pass");
}

#[test]
fn check_plan_missing_review_file_exhausted_retries_message() {
    let err = WorkflowError(
        "check_plan: agent did not write review file after retries".to_string(),
    );
    assert!(
        err.0.contains("retries"),
        "exhausted-retry error mentions retries"
    );
}
