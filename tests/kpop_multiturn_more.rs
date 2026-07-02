mod common;

use common::{MtStubPrompts, parse_kpop_want};
use malvin::MultiturnPrompt;
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_progression::{KpopMultiturnParams, KpopMultiturnState};

#[test]
fn mpc_plan_done_stops_without_second_prompt() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    let mpc_plan = tmp.path().join("mpc_plan.md");
    std::fs::write(&path, "## Step 1 — KPOP test\n").unwrap();
    std::fs::write(&mpc_plan, "DONE\n").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: path,
        mpc_plan_path: mpc_plan,
        max_hypotheses: 50,
    })
    .unwrap();
    assert!(state.next_prompt().expect("after done").is_none());
}

#[test]
fn single_kpop_block_uses_max_hypotheses_as_want() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mpc_plan = tmp.path().join("mpc_plan.md");
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: path,
        mpc_plan_path: mpc_plan,
        max_hypotheses: 10,
    })
    .unwrap();
    let first = state.next_prompt().expect("first");
    let Some(MultiturnPrompt::KpopBlock(s)) = first else {
        panic!("expected kpop block");
    };
    assert_eq!(parse_kpop_want(&s).expect("want"), 10);
    assert!(state.next_prompt().expect("second").is_none());
}
