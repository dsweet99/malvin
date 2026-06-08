use crate::acp::{
    AgentRetryOutcome, SESSION_NEW_INTERNAL_MAX_SPAWN_ATTEMPTS,
    agent_string_is_session_new_internal_error, plan_agent_retry,
};
use crate::support_paths::DEFAULT_MAX_ACP_RETRIES;
use std::time::Duration;

/// Tenacious ACP budget (`cli::loop_opts::TENACIOUS_MAX_ACP_RETRIES`).
const TENACIOUS_TEST_MAX_ATTEMPTS: u32 = 9999;

const SESSION_NEW_INTERNAL_MSG: &str =
    "ACP `session/new` failed: code=-32603; message=\"Internal error\"";

#[test]
fn session_new_internal_error_is_detected() {
    assert!(agent_string_is_session_new_internal_error(
        SESSION_NEW_INTERNAL_MSG
    ));
    assert!(agent_string_is_session_new_internal_error(
        "ACP session/new failed: Internal error"
    ));
    assert!(!agent_string_is_session_new_internal_error("session/new failed"));
    assert!(!agent_string_is_session_new_internal_error(
        "ACP `session/prompt` failed: code=-32603; message=\"Internal error\""
    ));
}

#[test]
fn session_new_internal_error_retries_with_bounded_backoff() {
    let out = plan_agent_retry(SESSION_NEW_INTERNAL_MSG, 1, DEFAULT_MAX_ACP_RETRIES).unwrap();
    match out {
        AgentRetryOutcome::Sleep(d) => assert_eq!(d, Duration::from_secs(1)),
        AgentRetryOutcome::StopRetrying => panic!("first spawn Internal should retry: {out:?}"),
    }
    let out = plan_agent_retry(SESSION_NEW_INTERNAL_MSG, 2, DEFAULT_MAX_ACP_RETRIES).unwrap();
    match out {
        AgentRetryOutcome::Sleep(d) => assert_eq!(d, Duration::from_secs(3)),
        AgentRetryOutcome::StopRetrying => panic!("second spawn Internal should retry: {out:?}"),
    }
}

#[test]
fn session_new_internal_error_stops_retrying() {
    let out = plan_agent_retry(
        SESSION_NEW_INTERNAL_MSG,
        SESSION_NEW_INTERNAL_MAX_SPAWN_ATTEMPTS,
        DEFAULT_MAX_ACP_RETRIES,
    )
    .unwrap();
    assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
}

#[test]
fn session_new_internal_error_not_retried_under_tenacious_budget() {
    let out = plan_agent_retry(SESSION_NEW_INTERNAL_MSG, 1, TENACIOUS_TEST_MAX_ATTEMPTS).unwrap();
    assert!(
        matches!(out, AgentRetryOutcome::Sleep(_)),
        "tenacious budget must not bypass spawn Internal micro-budget: {out:?}"
    );
    let out = plan_agent_retry(
        SESSION_NEW_INTERNAL_MSG,
        SESSION_NEW_INTERNAL_MAX_SPAWN_ATTEMPTS,
        TENACIOUS_TEST_MAX_ATTEMPTS,
    )
    .unwrap();
    assert!(
        matches!(out, AgentRetryOutcome::StopRetrying),
        "spawn Internal must stop at micro-budget even under tenacious: {out:?}"
    );
}
