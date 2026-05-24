#[test]
fn smoke_prompt_stdout_replacement_learn_vs_coder() {
    assert_eq!(
        crate::acp::prompt_stdout_replacement("learn"),
        Some(crate::output::LEARNING_PLACEHOLDER)
    );
    assert_eq!(crate::acp::prompt_stdout_replacement("coder"), None);
}
