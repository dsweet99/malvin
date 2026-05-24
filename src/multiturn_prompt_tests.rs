use crate::multiturn_prompt::MultiturnPrompt;

#[test]
fn multiturn_prompt_as_str() {
    let p = MultiturnPrompt::KpopBlock("k".into());
    assert_eq!(p.as_str(), "k");
}
