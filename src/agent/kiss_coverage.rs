
#[test]
fn kiss_cov_prompt_options() {
    use crate::acp::CoderPromptOptions;
    use crate::agent::PromptOptions;

    let opts = CoderPromptOptions {
        single_attempt: true,
        ..CoderPromptOptions::default()
    };
    let po = PromptOptions::from_coder(&opts);
    assert!(po.single_attempt);
    let _ = (
        stringify!(PromptOptions),
        stringify!(from_coder),
        stringify!(single_attempt),
    );
}
