// K-pop progression and prompt template smoke tests.

use std::collections::HashMap;

use rand::{rngs::StdRng, SeedableRng};

use crate::multiturn_prompt::MultiturnPrompt;

#[test]
fn smoke_kpop_acp_prompt() {
    assert!(crate::kpop_acp_prompt::kpop_creative_enabled(0.5));
    let _ = crate::kpop_acp_prompt::CREATIVE_MIN_INTERACTION;
}

#[test]
fn smoke_kpop_progression_and_multiturn() {
    let _ = crate::kpop_progression::KPOP_CATCHUP_CAP;
    let mean = crate::kpop_progression::block_mean_from_p_creative(0.5);
    assert!(mean.is_finite());
    let mut rng = StdRng::seed_from_u64(1);
    let _ = crate::kpop_progression::poisson_block_size(&mut rng, mean);

    let text = "## Step 1 — KPOP a\n";
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
        crate::kpop_multiturn_prompts::KpopMultiturnPrompts::StubMt(
            crate::MtStubPrompts,
        ),
        exp,
        10,
        0.5,
    )
    .expect("multiturn state");
    assert_eq!(
        state.exp_log_path().file_name().and_then(|s| s.to_str()),
        Some("exp.md")
    );

    match MultiturnPrompt::KpopBlock("z".into()) {
        MultiturnPrompt::KpopBlock(s) => assert_eq!(s, "z"),
        MultiturnPrompt::Mbc2(_) => panic!("expected kpop block variant"),
    }
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
