use std::path::PathBuf;

use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use crate::kpop_progression::{KpopMultiturnParams, KpopMultiturnState};

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
        state.next_prompt().expect("first prompt").is_some(),
        "first attempt should offer the KPop block"
    );

    state.reset_for_transport_retry();

    assert!(
        state.next_prompt().expect("retry prompt").is_some(),
        "transport retry must re-offer the KPop block after a failed attempt"
    );
}

#[test]
fn reset_for_transport_retry_clears_done_latch_set_by_prompt_sent() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path: PathBuf = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path,
        mpc_plan_path: tmp.path().join("mpc_plan.md"),
    })
    .expect("state");

    assert!(state.next_prompt().expect("first").is_some());
    assert!(
        state.next_prompt().expect("second without reset").is_none(),
        "without reset, prompt_sent prevents another prompt"
    );

    state.reset_for_transport_retry();
    assert!(
        state.next_prompt().expect("after reset").is_some(),
        "reset must clear prompt_sent and done latches"
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

    assert!(state.next_prompt().expect("first prompt").is_some());
    std::fs::write(&mpc_plan_path, "DONE\n").expect("simulate agent writing done before failure");
    state.reset_for_transport_retry();
    assert!(
        state.next_prompt().expect("retry prompt").is_some(),
        "transport retry must re-offer after stripping stale mpc plan DONE"
    );
    assert_eq!(
        std::fs::read_to_string(&mpc_plan_path).expect("read mpc plan"),
        ""
    );
}
