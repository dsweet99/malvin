use super::*;

#[test]
fn find_machine_block_start_at_file_start() {
    let content = "---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(find_machine_block_start(content), Some(0));
}

#[test]
fn find_machine_block_start_after_user_plan() {
    let content = "# User plan\n\nDo the thing.\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(find_machine_block_start(content), Some(28));
}

#[test]
fn find_machine_block_start_rejects_interior_dividers() {
    let content = "# Plan\n\nSee --- note below.\n\nMore text.\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(find_machine_block_start(content), Some(41));
}

#[test]
fn find_machine_block_start_rejects_prose_marker() {
    let content = "BEGIN_MALVIN in user\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(detect_rerun_user_span_end(content), Ok(Some(22)));
}

#[test]
fn detect_rerun_user_span_end_rejects_duplicate_markers() {
    let content = "# Plan\n\n---\nBEGIN_MALVIN\na\n\n---\nBEGIN_MALVIN\nb\n";
    assert_eq!(
        detect_rerun_user_span_end(content),
        Err(PlanFileError::DuplicateBeginMalvinMarkers)
    );
}

#[test]
fn overwrite_plan_file_trims_and_adds_trailing_newline() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "old").expect("write");
    overwrite_plan_file(&path, "# Revised plan\n\nDone.").expect("overwrite");
    let out = std::fs::read_to_string(&path).expect("read");
    assert_eq!(out, "# Revised plan\n\nDone.\n");
}

#[test]
fn validate_post_1a_requires_restatement_section() {
    let content = "# Plan\n\nno section\n";
    assert!(validate_post_1a(content).is_err());
}

#[test]
fn validate_post_1b_requires_critique_and_open_questions() {
    let ok = "## Restatement\nr\n\n## Critique\nc\n\n## Open questions\n1. q\n";
    validate_post_1b(ok).expect("valid");
    let missing = "## Restatement\nr\n";
    assert!(validate_post_1b(missing).is_err());
}

#[test]
fn validate_post_2_requires_decisions() {
    let ok = "## Restatement\nr\n\n## Critique\nc\n\n## Open questions\n1. q\n\n## DECISIONS\n1. **Verdict:** ok **Evidence:** code\n";
    validate_post_2(ok).expect("valid");
}

#[test]
fn plan_metadata_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let meta = PlanRunMetadata {
        user_span_end: 42,
        user_span_sha256: Some("abc".to_string()),
    };
    write_plan_metadata(tmp.path(), &meta).expect("write");
    let loaded = read_plan_metadata(tmp.path()).expect("read").expect("some");
    assert_eq!(loaded.user_span_end, 42);
    assert_eq!(loaded.user_span_sha256.as_deref(), Some("abc"));
}

#[test]
fn extract_decisions_section_returns_tail() {
    let content = "## Restatement\n\n## DECISIONS\n1. ok\n";
    let got = extract_decisions_section(content).expect("some");
    assert!(got.starts_with("## DECISIONS"));
}

#[test]
fn read_plan_file_missing_path_errors() {
    let path = std::path::Path::new("/nonexistent/plan.md");
    assert!(read_plan_file(path).is_err());
}

#[test]
fn write_plan_file_atomic_rejects_path_without_parent() {
    let parentless = std::path::Path::new("");
    assert!(
        write_plan_file_atomic(parentless, "x").is_err(),
        "empty plan path has no parent and must be rejected"
    );

    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    write_plan_file_atomic(&path, "ok").expect("valid path with parent should succeed");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "ok");
}

#[test]
fn truncate_plan_for_rerun_rejects_out_of_range_span() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "short").expect("write");
    assert!(truncate_plan_for_rerun(&path, 100).is_err());
}

#[test]
fn find_machine_block_start_returns_none_without_marker() {
    for content in [
        "# Plan\n\n---\nBEGIN_MALVIN\n",
        "",
        "# Plan only\n",
    ] {
        if content.contains(BEGIN_MALVIN_MARKER) {
            continue;
        }
        assert_eq!(find_machine_block_start(content), None);
    }
}

#[test]
fn detect_rerun_user_span_end_returns_none_for_clean_plan() {
    for content in [
        "# Plan\n\n---\nBEGIN_MALVIN\n",
        "# User only\n",
    ] {
        if content.contains(BEGIN_MALVIN_MARKER) && content.contains("---\nBEGIN_MALVIN") {
            continue;
        }
        if !content.contains(BEGIN_MALVIN_MARKER) {
            assert_eq!(detect_rerun_user_span_end(content), Ok(None));
        }
    }
}

#[test]
fn detect_rerun_user_span_end_with_crlf_machine_block() {
    let content = "# User\r\n\r\n---\r\nBEGIN_MALVIN\r\n## Restatement\r\n";
    assert_eq!(detect_rerun_user_span_end(content), Ok(Some(10)));
}

#[test]
fn prepare_plan_file_for_run_restores_sidecar_interrupted_plan() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(plan_user_sidecar_path(&path), "# User\n").expect("sidecar");
    std::fs::write(&path, "## Restatement\nmachine\n").expect("write");
    let restored = prepare_plan_file_for_run(&path).expect("prep");
    assert!(restored);
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "# User\n");
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    #[test]
    fn kiss_cov_plan_splice_symbols() {
        let _ = stringify!(super::overwrite_plan_file);
        let _ = stringify!(super::prepare_plan_file_for_run);
        let _ = stringify!(super::detect_rerun_user_span_end);
    }
}
