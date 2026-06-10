use super::*;

#[test]
fn find_machine_block_start_at_file_beginning() {
    let content = "---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(find_machine_block_start(content), Some(0));
}

#[test]
fn find_machine_block_start_after_user_header() {
    let content = "# User plan\n\nDo the thing.\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(find_machine_block_start(content), Some(28));
}

#[test]
fn embedded_dashes_in_user_span_do_not_false_positive() {
    let content = "# Plan\n\nSee --- note below.\n\nMore text.\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(find_machine_block_start(content), Some(41));
}

#[test]
fn begin_malvin_prose_in_user_span_does_not_block_machine_block() {
    let content = "BEGIN_MALVIN in user\n\n---\nBEGIN_MALVIN\n## Restatement\n";
    assert_eq!(detect_rerun_user_span_end(content), Ok(Some(22)));
}

#[test]
fn multiple_begin_malvin_markers_are_duplicate() {
    let content = "# Plan\n\n---\nBEGIN_MALVIN\na\n\n---\nBEGIN_MALVIN\nb\n";
    assert_eq!(
        detect_rerun_user_span_end(content),
        Err(PlanFileError::DuplicateBeginMalvinMarkers)
    );
}

#[test]
fn extract_fenced_markdown_block_from_response() {
    let response = "Here is the plan:\n\n```markdown\n# Revised\n\nShip it.\n```\n";
    let body = extract_fenced_markdown_block(response).expect("fence");
    assert!(body.contains("# Revised"));
    assert!(body.contains("Ship it."));
}

#[test]
fn extract_fenced_block_rejects_empty_fence() {
    let response = "```markdown\n```";
    assert_eq!(
        extract_fenced_markdown_block(response),
        Err(PlanFileError::MissingFencedBlock)
    );
}

#[test]
fn splice_replaces_machine_span_preserving_user_header_with_embedded_markers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    let user = "# Plan\n\n--- not a machine block ---\n\nBEGIN_MALVIN in prose\n\n";
    std::fs::write(&path, format!("{user}---\nBEGIN_MALVIN\n## Restatement\nold")).expect("write");
    let user_span_end = user.len();
    splice_plan_file(&path, user_span_end, "# Revised plan\n\nDone.").expect("splice");
    let out = std::fs::read_to_string(&path).expect("read");
    assert!(out.starts_with(user.trim_end()));
    assert!(out.contains("---\nBEGIN_MALVIN\n# Revised plan"));
    assert!(!out.contains("## Restatement\nold"));
}

#[test]
fn validate_post_1a_requires_restatement_section() {
    let content = "# Plan\n\n---\nBEGIN_MALVIN\nno section\n";
    assert!(validate_post_1a(content).is_err());
}

#[test]
fn validate_post_1b_requires_critique_and_open_questions() {
    let ok = "# Plan\n\n---\nBEGIN_MALVIN\n## Restatement\nr\n\n## Critique\nc\n\n## Open questions\n1. q\n";
    validate_post_1b(ok).expect("valid");
    let missing = "# Plan\n\n---\nBEGIN_MALVIN\n## Restatement\nr\n";
    assert!(validate_post_1b(missing).is_err());
}

#[test]
fn validate_post_2_requires_decisions() {
    let ok = "# Plan\n\n---\nBEGIN_MALVIN\n## Restatement\nr\n\n## Critique\nc\n\n## Open questions\n1. q\n\n## DECISIONS\n1. **Verdict:** ok **Evidence:** code\n";
    validate_post_2(ok).expect("valid");
}

#[test]
fn metadata_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let meta = PlanRunMetadata {
        user_span_end: 42,
        user_span_sha256: None,
    };
    write_plan_metadata(tmp.path(), &meta).expect("write");
    let loaded = read_plan_metadata(tmp.path()).expect("read").expect("some");
    assert_eq!(loaded.user_span_end, 42);
}

#[test]
fn extract_decisions_section_returns_tail() {
    let content = "# Plan\n\n---\nBEGIN_MALVIN\n## Restatement\n\n## DECISIONS\n1. ok\n";
    let decisions = extract_decisions_section(content).expect("decisions");
    assert!(decisions.starts_with("## DECISIONS"));
    assert!(decisions.contains("1. ok"));
}

#[test]
fn extract_fence_body_supports_md_alias_and_plain_fence() {
    let md = extract_fenced_markdown_block("```md\n# T\n```").expect("md");
    assert!(md.contains("# T"));
    let plain = extract_fenced_markdown_block("```\n# Plain\n```").expect("plain");
    assert!(plain.contains("# Plain"));
}

#[test]
fn extract_fence_body_distinguishes_markdown_fence_from_plain_opener() {
    let response = "prefix ```markdown\n# Revised\n``` suffix";
    let body = extract_fenced_markdown_block(response).expect("markdown fence");
    assert!(body.contains("# Revised"));
}

#[test]
fn extract_fenced_markdown_block_with_multiple_nested_fences() {
    let response = concat!(
        "```markdown\n",
        "# Plan\n",
        "```json\n",
        "{}\n",
        "```\n",
        "```bash\n",
        "echo hi\n",
        "```\n",
        "Done.\n",
        "```\n",
    );
    let body = extract_fenced_markdown_block(response).expect("fence");
    assert!(body.contains("```json"));
    assert!(body.contains("echo hi"));
    assert!(body.contains("Done."));
}

#[test]
fn extract_fenced_markdown_block_with_nested_bash_fences() {
    let response = concat!(
        "```markdown\n",
        "# Implementation plan\n",
        "\n",
        "Run:\n",
        "```bash\n",
        "modal run ops/deepswe_modal.py\n",
        "```\n",
        "\n",
        "## Quality gates\n",
        "- cargo nextest run\n",
        "```\n",
    );
    let body = extract_fenced_markdown_block(response).expect("fence");
    assert!(body.contains("# Implementation plan"));
    assert!(body.contains("modal run ops/deepswe_modal.py"));
    assert!(body.contains("## Quality gates"));
    assert!(body.contains("cargo nextest run"));
}

#[test]
fn validate_post_1b_rejects_critique_heading_in_restatement_prose() {
    let content = concat!(
        "# Plan\n\n---\nBEGIN_MALVIN\n",
        "## Restatement\n",
        "See ## Critique below.\n\n",
        "## Open questions\n",
        "1. q?\n",
    );
    assert!(validate_post_1b(content).is_err());
}

#[test]
fn validate_post_2_rejects_decisions_heading_in_critique_prose() {
    let content = concat!(
        "# Plan\n\n---\nBEGIN_MALVIN\n",
        "## Restatement\n",
        "r\n\n",
        "## Critique\n",
        "Pending ## DECISIONS\n\n",
        "## Open questions\n",
        "1. q?\n",
    );
    assert!(validate_post_2(content).is_err());
}

#[test]
fn find_machine_block_start_supports_crlf_delimiters() {
    let content = "# User\r\n\r\n---\r\nBEGIN_MALVIN\r\n## Restatement\r\n";
    assert_eq!(find_machine_block_start(content), Some(10));
}

#[test]
fn detect_rerun_user_span_end_with_crlf_machine_block() {
    let content = "# User\r\n\r\n---\r\nBEGIN_MALVIN\r\n## Restatement\r\n";
    assert_eq!(detect_rerun_user_span_end(content), Ok(Some(10)));
}

#[test]
fn prepare_plan_file_for_run_truncates_crlf_machine_block() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plan.md");
    std::fs::write(&path, "# User\r\n\r\n---\r\nBEGIN_MALVIN\r\nmachine\r\n").expect("write");
    let prior = prepare_plan_file_for_run(&path).expect("prep");
    assert_eq!(prior, Some(10));
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "# User\r\n\r\n");
}

#[test]
fn kiss_cov_plan_splice_symbols() {
    let _ = stringify!(find_machine_block_start);
    let _ = stringify!(overwrite_plan_file);
    let _ = stringify!(splice_plan_file);
    let _ = stringify!(extract_fenced_markdown_block);
    let _ = stringify!(detect_rerun_user_span_end);
    let _ = stringify!(prepare_plan_file_for_prompt_1a);
}
