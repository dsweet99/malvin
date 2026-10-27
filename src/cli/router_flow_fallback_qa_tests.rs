use super::super::{BRIDGE, run_models, with_bridge};

fn check_credit_error(message: &str) {
    check_model_switch(message, true);
}

fn check_model_switch(message: &str, expect_backup: bool) {
    with_bridge(|work| {
        let bridge = BRIDGE.replace(
            "'Insufficient funds: ' + model",
            &serde_json::to_string(message).unwrap(),
        );
        std::fs::write(work.join("bridge.mjs"), bridge).unwrap();
        let result = run_models(work, "cursor:poor,cursor:good");
        let attempts = std::fs::read_to_string(work.join("attempts")).unwrap();
        assert_eq!(result.is_ok(), expect_backup, "{message} {result:?}");
        if expect_backup {
            assert_eq!(attempts, "poor\ngood\n", "{message} {result:?}");
        } else {
            assert!(result.as_ref().unwrap_err().contains(message), "{result:?}");
            let stayed = attempts.lines().all(|line| line == "poor");
            assert!(
                stayed && attempts.lines().next().is_some(),
                "message={message:?}, attempts={attempts:?}, result={result:?}"
            );
        }
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

#[test]
fn reached_or_exhausted_account_limits_try_backup() {
    check_model_switch("You've hit your usage limit", true);
    check_model_switch("Billing hard limit has been reached", true);
    check_model_switch("Quota exhausted", true);
}

#[test]
fn credential_and_throughput_failures_stay_on_the_first_model() {
    check_model_switch("no credentials", false);
    check_model_switch("rate limit exceeded", false);
    check_model_switch(
        "Rate limit reached for requests per min. Limit: 20 / min.",
        false,
    );
}

#[test]
fn disk_quota_exceeded_must_not_try_backup() {
    check_model_switch("Disk quota exceeded (os error 122)", false);
}
