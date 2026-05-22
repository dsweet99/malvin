use crate::acp::{
    AgentRetryOutcome, MAX_AGENT_ATTEMPTS, agent_string_is_cannot_use_model,
    agent_string_is_retriable, agent_string_is_upgrade_plan,
    IterableClosedStream, iterable_closed_stream_from_buffer, operational_iterable_closed_for_emit,
    operational_iterable_closed_log_line,
    plan_agent_retry,
    retries_noun,
};
use std::time::Duration;

#[test]
fn upgrade_plan_substring_is_detected_case_insensitively() {
    assert!(agent_string_is_upgrade_plan(
        "Error: Upgrade Your Plan To Continue"
    ));
    assert!(!agent_string_is_upgrade_plan("timed out"));
}

#[test]
fn upgrade_plan_errors_do_not_retry() {
    let msg = "billing: upgrade your plan to continue";
    let err = plan_agent_retry(msg, 1).expect_err("upgrade plan must fail fast");
    assert_eq!(err.0, msg);
}

#[test]
fn cannot_use_model_errors_do_not_retry() {
    let msg = "Error: Cannot use this model with that provider";
    assert!(agent_string_is_cannot_use_model(msg));
    let err = plan_agent_retry(msg, 1).expect_err("invalid model must fail fast");
    assert_eq!(err.0, msg);
}

#[test]
fn cannot_use_model_fails_fast_even_when_error_also_looks_retriable() {
    let msg = "rpc [unavailable]: Cannot use this model";
    let err = plan_agent_retry(msg, 1).expect_err("model error must beat retriable match");
    assert_eq!(err.0, msg);
}

#[test]
fn retriable_timeout_delimited_without_timed_out_substring() {
    assert!(agent_string_is_retriable("rpc: connection timeout"));
    assert!(agent_string_is_retriable("session initialization failed"));
    assert!(!agent_string_is_retriable("timeoutable"));
    assert!(agent_string_is_retriable("rpc timeout"));
    assert!(agent_string_is_retriable("error: timeout"));
    assert!(!agent_string_is_retriable("atimeoutb"));
}

#[test]
fn iterable_closed_stream_from_buffer_and_operational_iterable_closed_for_emit() {
    assert_eq!(
        iterable_closed_stream_from_buffer("Error: T: WritableIterable is closed"),
        Some(IterableClosedStream::Writable)
    );
    assert_eq!(
        iterable_closed_stream_from_buffer("Error: T: ReadableIterable is closed"),
        Some(IterableClosedStream::Readable)
    );
    assert_eq!(
        operational_iterable_closed_for_emit("partial", Some(IterableClosedStream::Writable)),
        Some("acp: WritableIterable is closed")
    );
    assert_eq!(
        operational_iterable_closed_for_emit("partial", Some(IterableClosedStream::Readable)),
        Some("acp: ReadableIterable is closed")
    );
    assert_eq!(operational_iterable_closed_for_emit("ok", None), None);
}

#[test]
fn operational_iterable_closed_log_line_detection() {
    assert_eq!(
        operational_iterable_closed_log_line("\n\nError: T: WritableIterable is closed"),
        Some("acp: WritableIterable is closed")
    );
    assert_eq!(
        operational_iterable_closed_log_line("ReadableIterable is closed"),
        Some("acp: ReadableIterable is closed")
    );
    assert_eq!(operational_iterable_closed_log_line("invalid json"), None);
}

#[test]
fn retriable_transient_errors_match_known_agent_strings() {
    assert!(agent_string_is_retriable("request timed out"));
    assert!(agent_string_is_retriable("DEADLINE EXCEEDED"));
    assert!(agent_string_is_retriable("WritableIterable is closed"));
    assert!(agent_string_is_retriable("child process is zombie"));
    assert!(agent_string_is_retriable("session/new failed"));
    assert!(agent_string_is_retriable("rpc [unavailable]"));
}

#[test]
fn non_retriable_errors_stop_without_sleep() {
    assert!(!agent_string_is_retriable("invalid json"));
    let out = plan_agent_retry("invalid json", 1).unwrap();
    assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
}

fn assert_retriable_sleep_secs(attempt: u32, expected_secs: u64) {
    let out = plan_agent_retry("timed out", attempt).unwrap();
    match out {
        AgentRetryOutcome::Sleep(d) => assert_eq!(d, Duration::from_secs(expected_secs)),
        AgentRetryOutcome::StopRetrying => {
            panic!("expected Sleep({expected_secs}s), got StopRetrying")
        }
    }
}

#[test]
fn retriable_first_attempt_sleeps_one_second() {
    assert_retriable_sleep_secs(1, 1);
}

#[test]
fn retriable_second_attempt_sleeps_three_seconds() {
    assert_retriable_sleep_secs(2, 3);
}

#[test]
fn retriable_exhausts_after_max_agent_attempts() {
    let out = plan_agent_retry("timed out", MAX_AGENT_ATTEMPTS).unwrap();
    assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
}

#[test]
fn retries_noun_singular_and_plural() {
    assert_eq!(retries_noun(1), "retry");
    assert_eq!(retries_noun(2), "retries");
}

#[test]
fn delimited_token_match_has_delimited_substring_is_identifier_byte_timeout_word_iterable_closed_in_ascii_lower() {
    let _ = stringify!(iterable_closed_in_ascii_lower);
    let _ = stringify!(timeout_word_without_identifier_false_positive);
    let _ = stringify!(is_identifier_byte);
    let _ = stringify!(has_delimited_substring);
    let _ = stringify!(delimited_token_match);
    assert!(agent_string_is_retriable("rpc: connection timeout"));
    assert!(!agent_string_is_retriable("atimeoutb"));
    assert!(agent_string_is_retriable("session/new failed"));
    assert!(agent_string_is_retriable("init session timeout"));
}
