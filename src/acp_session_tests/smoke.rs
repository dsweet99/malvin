#[test]
fn smoke_prompt_stdout_replacement_is_none() {
    assert_eq!(crate::acp::prompt_stdout_replacement("learn"), None);
    assert_eq!(crate::acp::prompt_stdout_replacement("coder"), None);
}
