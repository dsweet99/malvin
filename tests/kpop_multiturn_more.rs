mod common;

use malvin::MtStubPrompts;
use malvin::MultiturnPrompt;
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_progression::{KpopMultiturnParams, KpopMultiturnState};

#[test]
fn single_prompt_then_stop() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "## Step 1 — KPOP test\n").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: path,
    })
    .unwrap();
    let first = state.next_prompt().expect("kpop prompt");
    let Some(MultiturnPrompt::KpopBlock(s)) = first else {
        panic!("expected kpop block prompt");
    };
    assert!(s.contains("stub kpop block"));
    assert!(state.next_prompt().expect("after prompt").is_none());
}

#[test]
fn empty_exp_log_still_offers_single_prompt() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: path,
    })
    .unwrap();
    assert!(state.next_prompt().expect("kpop prompt").is_some());
    assert!(state.next_prompt().expect("after prompt").is_none());
}
