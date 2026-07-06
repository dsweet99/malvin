mod common;

use common::MtStubPrompts;
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
    })
    .unwrap();
    assert!(state.next_prompt().expect("after done").is_none());
}

#[test]
fn three_phase_prompts_then_stop() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mpc_plan = tmp.path().join("mpc_plan.md");
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: path,
        mpc_plan_path: mpc_plan,
    })
    .unwrap();
    let first = state.next_prompt().expect("phase A");
    let Some(MultiturnPrompt::KpopBlock(s)) = first else {
        panic!("expected kpop block for phase A");
    };
    assert!(s.contains("stub kpop block"));
    assert!(state.next_prompt().expect("phase B").is_some());
    assert!(state.next_prompt().expect("phase C").is_some());
    assert!(state.next_prompt().expect("after all phases").is_none());
}
