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
        max_hypotheses: 10,
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
        max_hypotheses: 10,
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
fn transport_retry_strips_stale_kpop_solved_and_reoffers_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp_log_path = tmp.path().join("exp_log.md");
    std::fs::write(&exp_log_path, "\n").expect("write exp log");

    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::Smoke(SmokeKpopBuilder),
        exp_log_path: exp_log_path.clone(),
        max_hypotheses: 10,
    })
    .expect("state");

    assert!(
        state.next_prompt().expect("first prompt").is_some(),
        "first attempt should offer the KPop block"
    );

    std::fs::write(
        &exp_log_path,
        "## Step 1 — KPOP stale\n## KPOP_SOLVED\npremature\n",
    )
    .expect("simulate agent writing solved before transport failure");

    state.reset_for_transport_retry();

    assert!(
        state.next_prompt().expect("retry prompt").is_some(),
        "transport retry must re-offer the KPop block after stripping stale KPOP_SOLVED"
    );
    let text = std::fs::read_to_string(&exp_log_path).expect("read exp log");
    assert!(
        !text.contains("## KPOP_SOLVED"),
        "stale KPOP_SOLVED must be removed from disk: {text:?}"
    );
}
