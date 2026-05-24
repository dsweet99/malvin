use malvin::{DEFAULT_LEARN_MIN_ELAPSED_MS, should_run_learn_check};

#[test]
fn should_run_learn_check_covers_learn_gate_module() {
    let _ = should_run_learn_check;
    assert!(should_run_learn_check(0, 0));
    assert!(!should_run_learn_check(60_000, 1));
    assert_eq!(DEFAULT_LEARN_MIN_ELAPSED_MS, 300_000);
}
