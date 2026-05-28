use std::collections::HashMap;

use crate::multiturn_prompt::MultiturnPrompt;

#[test]
fn smoke_kpop_progression_and_multiturn() {
    let text = "## Step 1 — KPop a\n";
    assert_eq!(crate::kpop_progression::count_kpop_entries(text), 1);
    assert_eq!(crate::kpop_progression::count_mbc2_entries(text), 0);
    assert_eq!(crate::kpop_progression::hypotheses_emitted(text), 1);
    assert!(!crate::kpop_progression::agent_declared_success(text));

    let tmp = tempfile::tempdir().expect("tempdir");
    let exp = tmp.path().join("exp.md");
    std::fs::write(&exp, "hello").expect("exp");
    let got = crate::kpop_progression::read_exp_log_text(&exp).expect("read exp");
    assert_eq!(got, "hello");

    let state = crate::kpop_progression::KpopMultiturnState::new(
        crate::kpop_multiturn_prompts::KpopMultiturnPrompts::StubMt(crate::MtStubPrompts),
        exp,
        10,
    )
    .expect("multiturn state");
    assert_eq!(
        state.exp_log_path().file_name().and_then(|s| s.to_str()),
        Some("exp.md")
    );

    let MultiturnPrompt::KpopBlock(s) = MultiturnPrompt::KpopBlock("z".into());
    assert_eq!(s, "z");
}

#[test]
fn smoke_prompts_template_surface() {
    crate::prompts::enforce_no_unresolved_braces("no braces").expect("ok");
    let mut ctx = HashMap::new();
    ctx.insert("k".to_string(), "v".to_string());
    let one = crate::prompts::render_template("{{ k }}", &ctx);
    assert_eq!(one, "v");
    assert_eq!(crate::prompts::substitute_template("x", &ctx), "x");
}
