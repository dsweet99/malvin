#[test]
fn smoke_format_prompt_path_relative() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path();
    let child = base.join("plan.md");
    std::fs::write(&child, "x").expect("write");
    let formatted = crate::workflow_context::format_prompt_path(&child, base);
    assert!(formatted.starts_with("./"));
}
