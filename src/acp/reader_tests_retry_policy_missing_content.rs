use crate::acp::{
    AgentRetryOutcome, agent_string_is_openrouter_missing_content, plan_agent_retry,
};

#[test]
fn missing_content_is_detected_case_insensitively() {
    assert!(agent_string_is_openrouter_missing_content(
        "mini HTTP failed after 1 transport attempts (limit 3): OpenRouter response missing assistant content"
    ));
    assert!(!agent_string_is_openrouter_missing_content("timed out"));
}

#[test]
fn missing_content_fails_fast_without_retry_sleep() {
    let msg = "mini HTTP failed after 1 transport attempts (limit 3): OpenRouter response missing assistant content";
    let err = plan_agent_retry(msg, 1, 99).expect_err("must fail fast");
    assert_eq!(err.0, msg);
    assert!(matches!(
        plan_agent_retry("rate limited", 1, 99).expect("retryable"),
        AgentRetryOutcome::Sleep(_)
    ));
}
