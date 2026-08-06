use crate::acp::{
    agent_error_requires_coder_session_teardown, agent_string_is_cursor_http2_transport_error,
    agent_string_is_stale_cursor_sdk_auth, cursor_http2_transport_error_message,
};

#[test]
fn child_health_transport_errors_require_coder_session_teardown() {
    for msg in [
        "acp child process appears hung",
        "acp child process is not running",
        "acp child process is zombie",
        "acp stdout closed",
        "acp: WritableIterable is closed",
        "Error: T: Connection stalled",
        "Error: RetriableError: [unavailable] PING timed out",
        "Error: RetriableError: [canceled] http/2 stream closed with error code CANCEL (0x8)",
        "Authentication",
        "AuthenticationError: If you are logged in, try logging out and back in.",
        "ERROR_NOT_LOGGED_IN",
        "[unauthenticated] Error",
    ] {
        assert!(
            agent_error_requires_coder_session_teardown(msg),
            "{msg}"
        );
    }
    assert!(!agent_error_requires_coder_session_teardown("request timed out"));
}

#[test]
fn mock_agent_mode_keeps_session_after_ping_retriable_error() {
    crate::test_utils::enable_test_fast_teardown();
    assert!(!agent_error_requires_coder_session_teardown(
        "Error: RetriableError: [unavailable] PING timed out"
    ));
    assert!(agent_error_requires_coder_session_teardown(
        "acp child process is not running"
    ));
}

#[test]
fn cursor_http2_transport_errors_are_detected_and_normalized() {
    assert!(agent_string_is_cursor_http2_transport_error(
        "Error: RetriableError: [unavailable] PING timed out"
    ));
    assert_eq!(
        cursor_http2_transport_error_message(
            "Error: RetriableError: [unavailable] PING timed out"
        ),
        Some("RetriableError: [unavailable] PING timed out")
    );
    assert!(agent_string_is_cursor_http2_transport_error(
        "Error: RetriableError: [canceled] http/2 stream closed with error code CANCEL (0x8)"
    ));
    assert_eq!(
        cursor_http2_transport_error_message(
            "\n\nError: RetriableError: [canceled] http/2 stream closed with error code CANCEL (0x8)"
        ),
        Some("RetriableError: [canceled] http/2 stream closed with error code CANCEL (0x8)")
    );
    assert!(!agent_string_is_cursor_http2_transport_error("wrote review_requirements.json"));
    assert!(cursor_http2_transport_error_message("ok").is_none());
}

#[test]
fn stale_cursor_sdk_auth_strings_are_detected() {
    assert!(agent_string_is_stale_cursor_sdk_auth("Authentication"));
    assert!(agent_string_is_stale_cursor_sdk_auth("ERROR_NOT_LOGGED_IN"));
    assert!(!agent_string_is_stale_cursor_sdk_auth("request timed out"));
}
