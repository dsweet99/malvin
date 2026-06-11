use super::*;

#[test]
fn read_write_plan_file_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    write_plan_file_atomic(&path, "# Plan\n").expect("write");
    assert_eq!(read_plan_file(&path).expect("read"), "# Plan\n");
}

#[test]
fn truncate_plan_for_rerun_keeps_user_prefix() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n\n---\nBEGIN_MALVIN\nold\n").expect("write");
    truncate_plan_for_rerun(&path, 8).expect("truncate");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "# User\n\n");
}

#[test]
fn restore_interrupted_plan_truncates_legacy_block_without_sidecar() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n\n---\nBEGIN_MALVIN\nmachine\n").expect("write");
    restore_interrupted_plan(&path).expect("restore");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "# User\n\n");
}

#[test]
fn plan_metadata_round_trip_with_optional_hash() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let meta = PlanRunMetadata {
        user_span_end: 99,
        user_span_sha256: Some("deadbeef".to_string()),
    };
    write_plan_metadata(tmp.path(), &meta).expect("write");
    let loaded = read_plan_metadata(tmp.path()).expect("read").expect("some");
    assert_eq!(loaded.user_span_sha256.as_deref(), Some("deadbeef"));
}

#[test]
fn read_plan_metadata_missing_file_returns_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(read_plan_metadata(tmp.path()).expect("read").is_none());
}

#[test]
fn overwrite_plan_file_empty_body_writes_newline_only_when_nonempty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    overwrite_plan_file(&path, "").expect("write");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "");
}

#[test]
fn snapshot_plan_artifact_copies_to_run_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n").expect("write");
    let dest = snapshot_plan_artifact(&run_dir, "plan.p1a.md", &path).expect("snap");
    assert_eq!(dest, run_dir.join("plan.p1a.md"));
    assert_eq!(
        std::fs::read_to_string(dest).expect("read"),
        "# User\n"
    );
}

#[test]
fn post_overwrite_file_valid_for_create_run_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n").expect("write");
    prepare_plan_file_for_prompt_1a(&path).expect("prep");
    let art = crate::artifacts::create_run_artifacts(&path, Some(tmp.path())).expect("artifacts");
    assert!(art.run_dir.join("plan.md").is_file());
}

#[test]
fn prepare_plan_file_for_prompt_1a_overwrites_entire_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n").expect("write");
    prepare_plan_file_for_prompt_1a(&path).expect("prep");
    let out = std::fs::read_to_string(&path).expect("read");
    assert_eq!(out, "## Restatement\n");
    assert_eq!(
        std::fs::read_to_string(plan_user_sidecar_path(&path)).expect("sidecar"),
        "# User\n"
    );
}

#[test]
fn validate_post_1a_accepts_restatement_only_file() {
    let content = "## Restatement\nr\n";
    validate_post_1a(content).expect("valid");
}

#[test]
fn overwrite_plan_file_from_empty_user_span() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "").expect("write");
    overwrite_plan_file(&path, "# Revised\n").expect("overwrite");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "# Revised\n");
}

#[test]
fn remove_plan_user_sidecar_is_noop_when_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    remove_plan_user_sidecar(&path).expect("noop");
}

#[test]
fn prepare_plan_file_for_run_truncates_legacy_machine_block() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\n\n---\nBEGIN_MALVIN\nmachine\n").expect("write");
    let prior = prepare_plan_file_for_run(&path).expect("prep");
    assert!(prior);
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "# User\n\n");
}

#[test]
fn detect_rerun_user_span_end_ignores_prose_marker() {
    let content = "# Plan\n\nSee BEGIN_MALVIN in docs.\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(detect_rerun_user_span_end(content), Ok(Some(35)));
}

#[test]
fn detect_rerun_user_span_end_handles_mixed_eol_markers() {
    let lf_crlf = "# User\n\n---\r\nBEGIN_MALVIN\r\n## Restatement\r\n";
    assert_eq!(detect_rerun_user_span_end(lf_crlf), Ok(Some(8)));
    let crlf_lf = "# User\r\n\r\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(detect_rerun_user_span_end(crlf_lf), Ok(Some(10)));
}

#[test]
fn detect_rerun_user_span_end_rejects_prose_then_real_marker() {
    let content = "# Plan\nBEGIN_MALVIN\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(
        detect_rerun_user_span_end(content),
        Err(PlanFileError::DuplicateBeginMalvinMarkers)
    );
}
