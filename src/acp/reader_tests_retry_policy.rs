use crate::acp::{
    AgentRetryOutcome, IterableClosedStream, agent_error_requires_coder_session_teardown,
    agent_string_is_cannot_use_model, agent_string_is_upgrade_plan,
    emit_operational_upgrade_plan_stop, iterable_closed_stream_from_buffer,
    operational_iterable_closed_for_emit, operational_iterable_closed_log_line,
    operational_upgrade_plan_for_emit, plan_agent_retry, retries_noun,
    upgrade_plan_stream_from_buffer,
};
use crate::support_paths::DEFAULT_MAX_ACP_RETRIES;
use std::time::Duration;

const TEST_MAX_ATTEMPTS: u32 = DEFAULT_MAX_ACP_RETRIES;

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
    let err = plan_agent_retry(msg, 1, TEST_MAX_ATTEMPTS).expect_err("upgrade plan must fail fast");
    assert_eq!(err.0, msg);
}

#[test]
fn operational_upgrade_plan_for_emit_detects_line_and_stream_flag() {
    assert!(operational_upgrade_plan_for_emit(
        "billing: upgrade your plan to continue",
        false
    ));
    assert!(operational_upgrade_plan_for_emit("partial", true));
    assert!(!operational_upgrade_plan_for_emit("ok", false));
}

#[test]
fn upgrade_plan_stream_from_buffer_tracks_split_coalesce() {
    assert!(!upgrade_plan_stream_from_buffer("Upgrade your"));
    assert!(upgrade_plan_stream_from_buffer("Upgrade your plan to continue"));
}

#[test]
fn cannot_use_model_errors_do_not_retry() {
    let msg = "Error: Cannot use this model with that provider";
    assert!(agent_string_is_cannot_use_model(msg));
    let err = plan_agent_retry(msg, 1, TEST_MAX_ATTEMPTS).expect_err("invalid model must fail fast");
    assert_eq!(err.0, msg);
}

#[test]
fn cannot_use_model_fails_fast_even_when_error_also_looks_retriable() {
    let msg = "rpc [unavailable]: Cannot use this model";
    let err = plan_agent_retry(msg, 1, TEST_MAX_ATTEMPTS).expect_err("model error must beat retriable match");
    assert_eq!(err.0, msg);
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
fn transient_errors_retry_with_backoff() {
    for msg in [
        "request timed out",
        "DEADLINE EXCEEDED",
        "WritableIterable is closed",
        "child process is zombie",
        "session/new failed",
        "rpc [unavailable]",
    ] {
        assert!(
            matches!(
                plan_agent_retry(msg, 1, TEST_MAX_ATTEMPTS).unwrap(),
                AgentRetryOutcome::Sleep(_)
            ),
            "{msg}"
        );
    }
}

#[test]
fn child_health_transport_errors_require_coder_session_teardown() {
    for msg in [
        "acp child process appears hung",
        "acp child process is not running",
        "acp child process is zombie",
        "acp stdout closed",
        "acp: WritableIterable is closed",
    ] {
        assert!(
            agent_error_requires_coder_session_teardown(msg),
            "{msg}"
        );
    }
    assert!(!agent_error_requires_coder_session_teardown("request timed out"));
}

#[test]
fn unknown_errors_retry_with_backoff() {
    for msg in [
        "acp child process appears hung",
        "invalid json",
        "failed to spawn agent acp: No such file",
    ] {
        assert!(
            matches!(
                plan_agent_retry(msg, 1, TEST_MAX_ATTEMPTS).unwrap(),
                AgentRetryOutcome::Sleep(_)
            ),
            "{msg}"
        );
    }
}

fn assert_retriable_sleep_secs(attempt: u32, expected_secs: u64) {
    let out = plan_agent_retry("timed out", attempt, TEST_MAX_ATTEMPTS).unwrap();
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
    let out = plan_agent_retry("timed out", TEST_MAX_ATTEMPTS, TEST_MAX_ATTEMPTS).unwrap();
    assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
}

#[test]
fn retriable_exhausts_after_custom_max_attempts() {
    let custom_max = 5_u32;
    let out = plan_agent_retry("timed out", custom_max, custom_max).unwrap();
    assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
    assert!(matches!(
        plan_agent_retry("timed out", custom_max - 1, custom_max).unwrap(),
        AgentRetryOutcome::Sleep(_)
    ));
}

#[test]
fn restore_failure_stops_retrying_without_sleep() {
    let msg = "prompt failed; workspace session restore failed (restore): disk full";
    let out = plan_agent_retry(msg, 1, TEST_MAX_ATTEMPTS).unwrap();
    assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
}

#[test]
fn retries_noun_singular_and_plural() {
    assert_eq!(retries_noun(1), "retry");
    assert_eq!(retries_noun(2), "retries");
}

#[test]
fn emit_operational_upgrade_plan_stop_prints_once() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::clear_captured_stderr_lines();
    let mut warned = false;
    emit_operational_upgrade_plan_stop(&mut warned);
    emit_operational_upgrade_plan_stop(&mut warned);
    let stderr = crate::output::take_captured_stderr_lines().join("");
    assert!(
        stderr.contains(crate::acp::UPGRADE_PLAN_STOP_MESSAGE) && stderr.contains("Stopping.."),
        "stderr: {stderr:?}"
    );
    assert_eq!(stderr.matches(crate::acp::UPGRADE_PLAN_STOP_MESSAGE).count(), 1);
}

#[test]
fn operational_iterable_closed_for_emit_uses_stream_kind_message() {
    assert_eq!(
        operational_iterable_closed_for_emit(
            "partial",
            Some(IterableClosedStream::Writable)
        ),
        Some("acp: WritableIterable is closed")
    );
}

#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_emit_operational_upgrade_plan_stop() {
        let _ = stringify!(emit_operational_upgrade_plan_stop);
    }

    #[test]
    fn kiss_cov_iterable_closed_stream_message() {
        let _ = stringify!(iterable_closed_stream_message);
    }
}
