use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use crate::kpop_progression::{KpopMultiturnParams, KpopMultiturnState};
use crate::kpop_test_stubs::CaptureWants;
use std::sync::{Arc, Mutex};

const TEST_MAX_HYPOTHESES: usize = 10;

#[test]
fn kpop_multiturn_transport_retry_offers_prompt_again_after_failed_attempt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path,
        max_hypotheses: TEST_MAX_HYPOTHESES,
    })
    .expect("state");

    assert!(
        state.next_prompt().expect("first prompt").is_some(),
        "first attempt should offer the kpop prompt"
    );

    state.reset_for_transport_retry();

    assert!(
        state.next_prompt().expect("retry prompt").is_some(),
        "transport retry must re-offer the kpop prompt after a failed attempt"
    );
}

#[test]
fn single_prompt_then_done() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path,
        max_hypotheses: TEST_MAX_HYPOTHESES,
    })
    .expect("state");

    assert!(state.next_prompt().expect("kpop prompt").is_some());
    assert!(
        state.next_prompt().expect("after prompt").is_none(),
        "after the single kpop prompt, no more prompts should be offered"
    );
}

#[test]
fn capture_wants_records_one_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");
    let wants = Arc::new(Mutex::new(Vec::new()));

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubCapture(CaptureWants::new(wants.clone())),
        exp_log_path,
        max_hypotheses: TEST_MAX_HYPOTHESES,
    })
    .expect("state");

    assert!(state.next_prompt().expect("kpop prompt").is_some());
    assert!(state.next_prompt().expect("after prompt").is_none());
    assert_eq!(
        wants.lock().expect("wants lock").len(),
        1,
        "CaptureWants should record one kpop prompt"
    );
}

#[test]
fn reset_for_transport_retry_reoffers_single_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path,
        max_hypotheses: TEST_MAX_HYPOTHESES,
    })
    .expect("state");

    assert!(state.next_prompt().expect("kpop prompt").is_some());
    assert!(
        state.next_prompt().expect("after prompt").is_none(),
        "after the prompt, no more prompts"
    );

    state.reset_for_transport_retry();
    assert!(
        state.next_prompt().expect("after reset").is_some(),
        "reset must allow the kpop prompt to be offered again"
    );
}
