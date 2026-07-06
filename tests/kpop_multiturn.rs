//! Integration-style exercises for multiturn `KPop` state: simulate the agent by appending to the exp log
//! between `next_prompt` calls (no real `agent acp` child).

mod common;

use common::append_kpop_line;
use malvin::MultiturnPrompt;
use common::setup_multiturn_stub_mt;
use malvin::kpop_progression::hypotheses_emitted;

#[test]
fn kpop_single_prompt_then_stop_even_after_agent_writes_steps() {
    let common::MultiturnTestHarness { mut state, exp_path, _tmp } = setup_multiturn_stub_mt();
    let first = state.next_prompt().expect("kpop prompt");
    let MultiturnPrompt::KpopBlock(s) = first.expect("kpop prompt some");
    assert!(s.contains("stub kpop block"));
    for step in 1..=10 {
        append_kpop_line(&exp_path, step);
    }
    let p2 = state.next_prompt().expect("after prompt");
    assert!(p2.is_none(), "no second prompt after the single kpop turn");
    assert!(hypotheses_emitted(&std::fs::read_to_string(&exp_path).unwrap()) >= 10);
}
