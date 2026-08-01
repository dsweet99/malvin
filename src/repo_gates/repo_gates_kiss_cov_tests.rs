use super::prompt_quality_gates_markdown;

#[test]
fn kiss_cov_prompt_quality_gates_markdown_ok_path() {
    crate::test_utils::with_isolated_home(|w| {
        std::fs::create_dir_all(w.join(".malvin")).unwrap();
        std::fs::write(w.join(".malvin/checks"), "make lint\n").unwrap();
        let md = prompt_quality_gates_markdown(w).unwrap();
        assert!(md.contains("make lint"));
    });
    let _ = stringify!(prompt_quality_gates_markdown);
}
