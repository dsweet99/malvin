use std::collections::HashMap;

#[test]
fn kiss_stringify_tidy_helpers() {
    let _ = stringify!(super::TidyPromptRestore);
    let _ = stringify!(super::run_tidy_prompt_with_restore);
    let _ = stringify!(super::compose_tidy_concerns_prompt);
    let _ = stringify!(super::write_checks_do_not_pass_to_review_path);
    let _ = stringify!(super::run_tidy_interleaved_loop);
    let _ = stringify!(super::run_review_tidy_turn);
}

#[test]
fn write_checks_do_not_pass_writes_marker_line() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let p = tmp.path().join("nested").join("review.md");
    super::write_checks_do_not_pass_to_review_path(&p).expect("write");
    assert_eq!(
        std::fs::read_to_string(&p).expect("read"),
        "Checks do not pass\n"
    );
}

#[test]
fn compose_tidy_concerns_includes_review_path_when_present_in_context() {
    let store = malvin::prompts::PromptStore::default_store();
    let mut ctx = HashMap::new();
    ctx.insert("memories".to_string(), String::new());
    ctx.insert(
        "quality_gates_log".to_string(),
        "./_malvin/run/quality_gates.log".to_string(),
    );
    ctx.insert(
        "quality_gates".to_string(),
        "- `kiss check`\n".to_string(),
    );
    ctx.insert("plan_path".to_string(), "./plan.md".to_string());
    ctx.insert("review_path".to_string(), "./_malvin/run/review.md".to_string());
    let out = super::compose_tidy_concerns_prompt(&store, &ctx).expect("compose");
    assert!(
        out.contains("./_malvin/run/review.md"),
        "expected rendered concerns to cite review_path: {out:?}"
    );
}
