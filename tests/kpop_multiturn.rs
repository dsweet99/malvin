//! Integration-style exercises for multiturn `KPop` state: simulate the agent by appending to the exp log
//! between `next_prompt` calls (no real `agent acp` child).

mod common;

use common::{MtStubPrompts, append_kpop_line};
use malvin::KpopEchoPrompts;
use malvin::MultiturnPrompt;
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_progression::{KpopMultiturnParams, KpopMultiturnState, hypotheses_emitted};

#[test]
fn multiturn_exits_when_mpc_plan_hits_done() {
    let tmp = tempfile::tempdir().unwrap();
    let exp = tmp.path().join("exp.md");
    let mpc_plan = tmp.path().join("mpc_plan.md");
    std::fs::write(&exp, "\n").unwrap();
    std::fs::write(&mpc_plan, "DONE\n").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubEcho(KpopEchoPrompts),
        exp_log_path: exp,
        mpc_plan_path: mpc_plan,
    })
    .unwrap();
    assert!(state.next_prompt().unwrap().is_none());
}

#[test]
fn kpop_single_prompt_then_stop_even_after_agent_writes_steps() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mpc_plan = tmp.path().join("mpc_plan.md");
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: path.clone(),
        mpc_plan_path: mpc_plan,
    })
    .unwrap();
    let first = state.next_prompt().expect("prompt");
    let MultiturnPrompt::KpopBlock(s) = first.expect("first");
    assert!(s.contains("stub kpop block"));
    for step in 1..=10 {
        append_kpop_line(&path, step);
    }
    let p2 = state.next_prompt().expect("second");
    assert!(p2.is_none());
    assert!(hypotheses_emitted(&std::fs::read_to_string(&path).unwrap()) >= 10);
}
