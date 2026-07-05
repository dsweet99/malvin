use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use crate::kpop_progression::{KpopMultiturnParams, KpopMultiturnState};
use crate::kpop_test_stubs::CaptureBlocks;
use std::sync::{Arc, Mutex};

#[test]
fn kpop_multiturn_transport_retry_offers_prompt_again_after_failed_attempt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path,
        mpc_plan_path: tmp.path().join("mpc_plan.md"),
    })
    .expect("state");

    assert!(
        state.next_prompt().expect("phase Priors").is_some(),
        "first attempt should offer phase Priors"
    );

    state.reset_for_transport_retry();

    assert!(
        state.next_prompt().expect("retry phase Priors").is_some(),
        "transport retry must re-offer phase Priors after a failed attempt"
    );
}

#[test]
fn all_four_phases_are_offered_then_done() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path,
        mpc_plan_path: tmp.path().join("mpc_plan.md"),
    })
    .expect("state");

    assert!(state.next_prompt().expect("phase Priors").is_some());
    assert!(state.next_prompt().expect("phase A").is_some());
    assert!(state.next_prompt().expect("phase B").is_some());
    assert!(state.next_prompt().expect("phase C").is_some());
    assert!(
        state.next_prompt().expect("after all phases").is_none(),
        "after all four phases, no more prompts should be offered"
    );
}

#[test]
fn capture_blocks_records_priors_and_block_a_across_four_phases() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");
    let blocks = Arc::new(Mutex::new(Vec::new()));

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubCapture(CaptureBlocks::new(blocks.clone())),
        exp_log_path,
        mpc_plan_path: tmp.path().join("mpc_plan.md"),
    })
    .expect("state");

    assert!(state.next_prompt().expect("phase Priors").is_some());
    assert!(state.next_prompt().expect("phase A").is_some());
    assert!(state.next_prompt().expect("phase B").is_some());
    assert!(state.next_prompt().expect("phase C").is_some());
    assert!(state.next_prompt().expect("after all phases").is_none());
    assert_eq!(
        blocks.lock().expect("blocks lock").len(),
        2,
        "CaptureBlocks should record priors and block A completions"
    );
}

#[test]
fn reset_for_transport_retry_clears_phase_back_to_priors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path,
        mpc_plan_path: tmp.path().join("mpc_plan.md"),
    })
    .expect("state");

    assert!(state.next_prompt().expect("phase Priors").is_some());
    assert!(state.next_prompt().expect("phase A").is_some());
    assert!(state.next_prompt().expect("phase B").is_some());
    assert!(state.next_prompt().expect("phase C").is_some());
    assert!(
        state.next_prompt().expect("after all phases").is_none(),
        "after all phases, no more prompts"
    );

    state.reset_for_transport_retry();
    assert!(
        state.next_prompt().expect("after reset").is_some(),
        "reset must clear phase back to Priors"
    );
}

#[test]
fn transport_retry_strips_stale_mpc_plan_done_and_reoffers_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    let mpc_plan_path = tmp.path().join("mpc_plan.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");
    std::fs::write(&mpc_plan_path, "").expect("write mpc plan");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path: exp_log_path.clone(),
        mpc_plan_path: mpc_plan_path.clone(),
    })
    .expect("state");

    assert!(state.next_prompt().expect("phase Priors").is_some());
    std::fs::write(&mpc_plan_path, "DONE\n").expect("simulate agent writing done before failure");
    state.reset_for_transport_retry();
    assert!(
        state.next_prompt().expect("retry phase Priors").is_some(),
        "transport retry must re-offer after stripping stale mpc plan DONE"
    );
    assert_eq!(
        std::fs::read_to_string(&mpc_plan_path).expect("read mpc plan"),
        ""
    );
}

#[test]
fn done_check_between_b_and_c_stops_early() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    let mpc_plan_path = tmp.path().join("mpc_plan.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");
    std::fs::write(&mpc_plan_path, "").expect("write mpc plan");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path,
        mpc_plan_path: mpc_plan_path.clone(),
    })
    .expect("state");

    assert!(state.next_prompt().expect("phase Priors").is_some());
    assert!(state.next_prompt().expect("phase A").is_some());
    assert!(state.next_prompt().expect("phase B").is_some());
    std::fs::write(&mpc_plan_path, "DONE\n").expect("agent writes DONE after phase B");
    assert!(
        state.next_prompt().expect("after DONE").is_none(),
        "DONE written after phase B should prevent phase C"
    );
}
