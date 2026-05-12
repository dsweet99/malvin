#[test]
fn kiss_stringify_plan_flow_units() {
    let _ = stringify!(super::resolve_user_plan_path);
    let _ = stringify!(super::normalized_plan_file_bytes);
    let _ = stringify!(super::write_plan_source);
    let _ = stringify!(super::artifacts_work_dir_for_run);
    let _ = stringify!(super::plan_run_artifacts);
    let _ = stringify!(super::start_plan_workspace_session);
    let _ = stringify!(super::build_rendered_plan_prompt);
    let _ = stringify!(super::set_plan_timing_label);
    let _ = stringify!(super::restore_after_plan_prompt);
    let _ = stringify!(super::pair_run_and_restore);
    let _ = stringify!(super::plan_coder_prompt);
    let _ = stringify!(super::PlanReviewOnce);
    let _ = stringify!(super::run_plan_review_once);
    let _ = stringify!(super::run_plan);
}

#[test]
fn rejects_whitespace_only_plan_text() {
    assert!(super::normalized_plan_file_bytes(" \n\t ").is_err());
}

#[test]
fn preserves_leading_and_trailing_non_newline_whitespace() {
    let bytes = super::normalized_plan_file_bytes("  hi  ").expect("non-empty plan");
    assert_eq!(String::from_utf8(bytes).unwrap(), "  hi  \n");
}

#[test]
fn normalizes_trailing_newlines_to_single_terminal_newline() {
    let bytes = super::normalized_plan_file_bytes("a\n\n").expect("non-empty plan");
    assert_eq!(String::from_utf8(bytes).unwrap(), "a\n");
}
