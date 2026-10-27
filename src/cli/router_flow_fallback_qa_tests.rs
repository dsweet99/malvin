use super::super::{BRIDGE, run_models, with_bridge};

fn check_credit_error(message: &str) {
    with_bridge(|work| {
        let bridge = BRIDGE.replace(
            "'Insufficient funds: ' + model",
            &serde_json::to_string(message).unwrap(),
        );
        std::fs::write(work.join("bridge.mjs"), bridge).unwrap();
        let result = run_models(work, "cursor:poor,cursor:good");
        let attempts = std::fs::read_to_string(work.join("attempts")).unwrap();
        assert_eq!(
            attempts, "poor\ngood\n",
            "Credit exhaustion must try the backup; message={message:?}, result={result:?}"
        );
        assert!(result.is_ok(), "{result:?}");
    });
}

#[test]
fn recognized_credit_exhaustion_tries_backup_control() {
    check_credit_error("no credits remaining");
}

#[test]
fn short_credit_exhaustion_tries_backup() {
    check_credit_error("no credits");
}

#[test]
fn plural_insufficient_credits_tries_backup_control() {
    check_credit_error("Insufficient credits");
}

#[test]
fn singular_insufficient_credit_balance_tries_backup() {
    check_credit_error("Insufficient credit balance");
}

#[test]
fn usage_limit_exceeded_tries_backup() {
    check_credit_error("Usage limit exceeded");
}

#[test]
fn reached_usage_limit_tries_backup() {
    check_credit_error("Your usage limit has been reached");
}

#[test]
fn quota_billing_sentence_tries_backup() {
    check_credit_error(
        "You exceeded your current quota, please check your plan and billing details.",
    );
}

#[test]
fn plan_upgrade_without_continue_tries_backup() {
    check_credit_error("Upgrade your plan to use this model");
}

#[test]
fn more_credits_sentence_tries_backup() {
    check_credit_error("This request requires more credits, or fewer max_tokens.");
}

#[test]
fn singular_insufficient_credit_tries_backup() {
    check_credit_error("Insufficient credit.");
}
