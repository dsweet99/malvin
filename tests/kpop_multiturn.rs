//! Integration-style exercises for multiturn `KPop` state: simulate the agent by appending to the exp log
//! between `next_prompt` calls (no real `agent acp` child).

mod common;

use common::{MtStubPrompts, append_kpop_line, parse_kpop_want};
use malvin::KpopEchoPrompts;
use malvin::MultiturnPrompt;
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_progression::{KpopMultiturnParams, KpopMultiturnState, hypotheses_emitted};

#[test]
fn multiturn_stops_immediately_when_exp_log_already_at_max_hypotheses() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(
        &path,
        "## Step 1 — KPOP a\n## Step 2 — KPOP b\n## Step 3 — MBC2 c\n",
    )
    .unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubEcho(KpopEchoPrompts),
        exp_log_path: path,
        max_hypotheses: 3,
    })
    .unwrap();
    assert!(state.next_prompt().unwrap().is_none());
}

#[test]
fn multiturn_exits_when_exp_log_hits_success_marker() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "## KPOP_SOLVED\nx\n").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubEcho(KpopEchoPrompts),
        exp_log_path: tmp.path().to_path_buf(),
        max_hypotheses: 100,
    })
    .unwrap();
    assert!(state.next_prompt().unwrap().is_none());
}

#[test]
fn kpop_want_equals_max_hypotheses_in_single_prompt() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: path,
        max_hypotheses: 3,
    })
    .unwrap();
    let first = state.next_prompt().expect("prompt").expect("first");
    let MultiturnPrompt::KpopBlock(s) = first else {
        panic!("expected kpop block");
    };
    let want = parse_kpop_want(&s).expect("want");
    assert_eq!(want, 3);
}

#[test]
fn kpop_single_prompt_then_stop_even_after_agent_writes_steps() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: path.clone(),
        max_hypotheses: 50,
    })
    .unwrap();
    let first = state.next_prompt().expect("prompt");
    let MultiturnPrompt::KpopBlock(s) = first.expect("first") else {
        panic!("expected kpop block");
    };
    let want = parse_kpop_want(&s).expect("want in stub");
    for step in 1..=want {
        append_kpop_line(&path, step);
    }
    let p2 = state.next_prompt().expect("second");
    assert!(p2.is_none());
    assert!(hypotheses_emitted(&std::fs::read_to_string(&path).unwrap()) >= want);
}
