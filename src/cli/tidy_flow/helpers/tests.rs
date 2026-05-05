#![cfg(test)]

#[test]
fn kiss_stringify_tidy_helpers() {
    let _ = stringify!(super::TidyPromptRestore);
    let _ = stringify!(super::run_tidy_prompt_with_restore);
}
