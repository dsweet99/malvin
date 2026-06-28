use super::prompt_quality_gates_markdown;

#[test]
fn kiss_cov_prompt_quality_gates_markdown_ok_path() {
    crate::test_utils::with_isolated_home(|w| {
        std::fs::write(
            w.join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();
        super::ensure_default_malvin_checks_file(w).unwrap();
        let md = prompt_quality_gates_markdown(w).unwrap();
        assert!(md.contains("kiss check"));
    });
    let _ = stringify!(prompt_quality_gates_markdown);
}
