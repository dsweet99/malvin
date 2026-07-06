//! Integration-style exercises for multiturn `KPop` state: simulate the agent by appending to the exp log
//! between `next_prompt` calls (no real `agent acp` child).

mod common;

use common::append_kpop_line;
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
fn kpop_three_phases_then_stop_even_after_agent_writes_steps() {
    let common::MultiturnTestHarness { mut state, exp_path, _tmp } =
        common::setup_multiturn_stub_mt();
    let first = state.next_prompt().expect("phase A");
    let MultiturnPrompt::KpopBlock(s) = first.expect("phase A some");
    assert!(s.contains("stub kpop block"));
    for step in 1..=10 {
        append_kpop_line(&exp_path, step);
    }
    let p2 = state.next_prompt().expect("phase B");
    assert!(p2.is_some(), "phase B should be offered");
    let p3 = state.next_prompt().expect("phase C");
    assert!(p3.is_some(), "phase C should be offered");
    let p4 = state.next_prompt().expect("after all phases");
    assert!(p4.is_none(), "no more prompts after all three phases");
    assert!(hypotheses_emitted(&std::fs::read_to_string(&exp_path).unwrap()) >= 10);
}
