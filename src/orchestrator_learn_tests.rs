use crate::orchestrator::{should_run_learn_check, DEFAULT_LEARN_MIN_ELAPSED_MS};

const FIVE_MIN_MS: u64 = DEFAULT_LEARN_MIN_ELAPSED_MS;

#[test]
fn should_run_learn_check_zero_threshold_always_runs() {
    assert!(should_run_learn_check(0, 0), "0 threshold, 0 elapsed => run");
    assert!(should_run_learn_check(0, 1), "0 threshold, any elapsed => run");
    assert!(
        should_run_learn_check(0, FIVE_MIN_MS),
        "0 threshold, 5 min => run"
    );
}

#[test]
fn should_run_learn_check_below_threshold_skips() {
    assert!(
        !should_run_learn_check(FIVE_MIN_MS, 0),
        "5 min threshold, 0 elapsed => skip"
    );
    assert!(
        !should_run_learn_check(FIVE_MIN_MS, 299_999),
        "5 min threshold, just under => skip"
    );
}

#[test]
fn should_run_learn_check_at_or_above_threshold_runs() {
    assert!(
        should_run_learn_check(FIVE_MIN_MS, FIVE_MIN_MS),
        "5 min threshold, exactly 5 min => run"
    );
    assert!(
        should_run_learn_check(FIVE_MIN_MS, FIVE_MIN_MS + 1),
        "5 min threshold, just over => run"
    );
    assert!(
        should_run_learn_check(FIVE_MIN_MS, FIVE_MIN_MS * 2),
        "5 min threshold, 10 min => run"
    );
}
