use std::path::Path;

use super::*;

#[test]
fn read_and_write_plan_file_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    write_plan_file_atomic(&path, "# Plan\n").expect("write");
    let text = read_plan_file(&path).expect("read");
    assert_eq!(text, "# Plan\n");
}

#[test]
fn truncate_plan_for_rerun_keeps_user_span_only() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n\n---\nBEGIN_MALVIN\nmachine\n").expect("write");
    truncate_plan_for_rerun(&path, 8).expect("truncate");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "# User\n\n");
}

#[test]
fn record_user_span_end_after_1a_matches_detect() {
    let content = "# User\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(record_user_span_end_after_1a(content).expect("span"), 8);
}

#[test]
fn record_user_span_end_errors_without_machine_block() {
    assert!(record_user_span_end_after_1a("# User only\n").is_err());
}

#[test]
fn metadata_round_trip_includes_optional_hash() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let meta = PlanRunMetadata {
        user_span_end: 99,
        user_span_sha256: Some("abc".into()),
    };
    write_plan_metadata(tmp.path(), &meta).expect("write");
    let loaded = read_plan_metadata(tmp.path()).expect("read").expect("some");
    assert_eq!(loaded.user_span_sha256.as_deref(), Some("abc"));
}

#[test]
fn read_plan_metadata_missing_file_returns_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(read_plan_metadata(tmp.path()).expect("read").is_none());
}

#[test]
fn plan_file_io_error_is_invalid_input() {
    use super::plan_file_io_error::plan_file_io_error;
    let err = plan_file_io_error("bad plan");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("bad plan"));
}

#[test]
fn write_plan_file_atomic_rejects_empty_path() {
    let err = write_plan_file_atomic(Path::new(""), "x").expect_err("empty path");
    assert!(matches!(err, PlanFileError::Io(_)));
}

#[test]
fn splice_rejects_out_of_range_user_span_end() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "short").expect("write");
    assert!(splice_plan_file(&path, 100, "# x").is_err());
}

#[test]
fn section_present_after_marker_false_when_marker_absent() {
    let content = "# Plan\n\n## Restatement\nno marker\n";
    assert!(validate_post_1a(content).is_err());
}

#[test]
fn post_splice_file_valid_for_create_run_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n\n").expect("write");
    splice_plan_file(&path, 8, "# Revised\n\nSpec.").expect("splice");
    let art = crate::artifacts::create_run_artifacts(&path, Some(tmp.path())).expect("artifacts");
    assert!(art.plan_path.is_file());
}

#[test]
fn extract_fence_body_handles_crlf_after_fence_marker() {
    let body = extract_fenced_markdown_block("```markdown\r\n# CRLF\r\n```").expect("fence");
    assert!(body.contains("# CRLF"));
}

#[test]
fn splice_adds_newline_when_user_span_has_no_trailing_newline() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User").expect("write");
    splice_plan_file(&path, 6, "# Revised").expect("splice");
    let out = std::fs::read_to_string(&path).expect("read");
    assert!(out.contains("# User\n\n---\nBEGIN_MALVIN"));
}

#[test]
fn append_machine_block_trims_fenced_body_trailing_whitespace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n\n").expect("write");
    splice_plan_file(&path, 8, "# Revised  \n\n  ").expect("splice");
    let out = std::fs::read_to_string(&path).expect("read");
    assert!(out.contains("# Revised\n"));
    assert!(!out.contains("  \n\n---"));
}

#[test]
fn validate_post_1b_errors_when_critique_section_missing() {
    let content = "# Plan\n\n---\nBEGIN_MALVIN\n## Restatement\nr\n";
    assert!(validate_post_1b(content).is_err());
}

#[test]
fn extract_fence_body_uses_rfind_when_no_newline_before_close() {
    let body = extract_fenced_markdown_block("```\ninline```").expect("inline close");
    assert_eq!(body, "inline");
}

#[test]
fn splice_empty_user_span_still_writes_machine_block() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "").expect("write");
    splice_plan_file(&path, 0, "# Revised\n").expect("splice");
    let out = std::fs::read_to_string(&path).expect("read");
    assert!(out.contains("---\nBEGIN_MALVIN"));
}

#[test]
fn splice_preserves_fenced_body_trailing_newline() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n\n").expect("write");
    splice_plan_file(&path, 8, "# Revised\n").expect("splice");
    assert!(std::fs::read_to_string(&path).expect("read").ends_with("# Revised\n"));
}

#[test]
fn prepare_plan_file_for_run_truncates_machine_block() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n\n---\nBEGIN_MALVIN\nmachine\n").expect("write");
    let prior = prepare_plan_file_for_run(&path).expect("prep");
    assert_eq!(prior, Some(8));
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "# User\n\n");
}

#[test]
fn prose_begin_malvin_substring_does_not_count_as_marker_line() {
    let content = "# Plan\n\nSee BEGIN_MALVIN in docs.\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(detect_rerun_user_span_end(content), Ok(Some(35)));
}

#[test]
fn find_machine_block_start_supports_mixed_eol_delimiters() {
    let lf_crlf = "# User\n\n---\r\nBEGIN_MALVIN\r\n## Restatement\r\n";
    assert_eq!(find_machine_block_start(lf_crlf), Some(8));
    let crlf_lf = "# User\r\n\r\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(find_machine_block_start(crlf_lf), Some(10));
}

#[test]
fn whole_line_begin_malvin_in_user_span_is_duplicate() {
    let content = "# Plan\nBEGIN_MALVIN\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(
        detect_rerun_user_span_end(content),
        Err(PlanFileError::DuplicateBeginMalvinMarkers)
    );
}
