use crate::agent_process::{
    agent_error_requires_coder_session_teardown, agent_string_is_cursor_agent_busy,
    agent_string_is_cursor_http2_transport_error, agent_string_is_stale_cursor_sdk_auth,
};

#[test]
fn child_health_transport_errors_require_coder_session_teardown() {
    let _guard = crate::test_support::test_utils::test_env_lock();
    crate::test_support::test_utils::clear_test_no_real_agent_env();
    for msg in [
        "acp child process appears hung",
        "acp child process is not running",
        "acp child process is zombie",
        "acp stdout closed",
        "acp: WritableIterable is closed",
        "bridge drain timed out waiting for run_done after 1s without a bridge event (bridge quiet; likely hung or stalled)",
        "bridge timed out waiting for run_done after 1s without a bridge event (bridge quiet; likely hung or stalled)",
        "npm pi rpc timed out waiting for npm pi event after 45s of silence",
        "npm pi stdout closed",
        "npm pi write: broken pipe",
        "codex timed out waiting for turn event after 1s of silence",
        "codex stdout closed",
        "codex write: broken pipe",
        "codex flush: broken pipe",
        "codex read: connection reset",
        "codex JSON-RPC parse: expected value",
        "Agent is currently streaming; specify streamingBehavior",
        "Error: T: Connection stalled",
        "Error: RetriableError: [unavailable] PING timed out",
        "Error: RetriableError: [canceled] http/2 stream closed with error code CANCEL (0x8)",
        "Authentication",
        "AuthenticationError: If you are logged in, try logging out and back in.",
        "ERROR_NOT_LOGGED_IN",
        "[unauthenticated] Error",
        "Agent agent-7b61bfe2-fa7a-47bd-8f5b-96c158067bc8 already has active run",
    ] {
        assert!(agent_error_requires_coder_session_teardown(msg), "{msg}");
    }
    assert!(!agent_error_requires_coder_session_teardown(
        "request timed out"
    ));
}

#[test]
fn live_drain_idle_prefixes_require_coder_session_teardown() {
    for prefix in [
        crate::config::model_id::ModelBackend::Cursor.drain_idle_prefix(),
        crate::config::model_id::ModelBackend::Pi.drain_idle_prefix(),
        crate::config::model_id::ModelBackend::Codex.drain_idle_prefix(),
    ] {
        let msg = format!(
            "{prefix} waiting for event after 1s without a bridge event (bridge quiet; likely hung or stalled)"
        );
        assert!(agent_error_requires_coder_session_teardown(&msg), "{msg}");
    }
}

#[test]
fn per_backend_wire_failures_are_session_dead() {
    for msg in [
        "bridge stdout closed",
        "bridge write: x",
        "bridge flush: x",
        "bridge read: x",
        "npm pi stdout closed",
        "npm pi write: x",
        "npm pi flush: x",
        "npm pi read: x",
        "npm pi JSONL parse: x",
        "codex stdout closed",
        "codex write: x",
        "codex flush: x",
        "codex read: x",
        "codex JSON-RPC parse: x",
    ] {
        assert_eq!(
            crate::agent_process::classify_agent_fault(msg),
            crate::agent_process::AgentFault::SessionDead,
            "{msg}"
        );
    }
    assert_eq!(
        crate::agent_process::classify_agent_fault("bridge event parse: x"),
        crate::agent_process::AgentFault::Ordinary
    );
}

#[test]
fn every_backend_marker_and_parse_error_is_session_dead() {
    let parse_err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    for backend in crate::config::model_id::ModelBackend::ALL {
        for marker in backend.spec().dead_session_markers() {
            assert_eq!(
                crate::agent_process::classify_agent_fault(&format!("{marker} x")),
                crate::agent_process::AgentFault::SessionDead,
                "{marker}"
            );
        }
        if backend.spec().wire_parse_label().is_some() {
            let msg = crate::backends::bridge_sdk::json_parse_error(backend, &parse_err);
            assert_eq!(
                crate::agent_process::classify_agent_fault(&msg),
                crate::agent_process::AgentFault::SessionDead,
                "{msg}"
            );
        }
    }
}

#[test]
fn cursor_agent_busy_strings_are_detected() {
    assert!(agent_string_is_cursor_agent_busy(
        "Agent agent-7b61bfe2-fa7a-47bd-8f5b-96c158067bc8 already has active run"
    ));
    assert!(agent_string_is_cursor_agent_busy("ALREADY HAS ACTIVE RUN"));
    assert!(!agent_string_is_cursor_agent_busy("bridge stdout closed"));
}

#[test]
fn mock_agent_mode_keeps_session_after_ping_retriable_error() {
    let _guard = crate::test_support::test_utils::test_env_lock();
    crate::test_support::test_utils::enable_test_fast_teardown();
    assert!(!agent_error_requires_coder_session_teardown(
        "Error: RetriableError: [unavailable] PING timed out"
    ));
    assert!(agent_error_requires_coder_session_teardown(
        "acp child process is not running"
    ));
    crate::test_support::test_utils::clear_test_no_real_agent_env();
}

#[test]
fn cursor_http2_transport_errors_are_detected() {
    assert!(agent_string_is_cursor_http2_transport_error(
        "Error: RetriableError: [unavailable] PING timed out"
    ));
    assert!(agent_string_is_cursor_http2_transport_error(
        "Error: RetriableError: [canceled] http/2 stream closed with error code CANCEL (0x8)"
    ));
    assert!(agent_string_is_cursor_http2_transport_error(
        "\n\nError: RetriableError: [canceled] http/2 stream closed with error code CANCEL (0x8)"
    ));
    assert!(!agent_string_is_cursor_http2_transport_error(
        "wrote review_requirements.json"
    ));
    assert!(!agent_string_is_cursor_http2_transport_error("ok"));
}

#[test]
fn stale_cursor_sdk_auth_strings_are_detected() {
    assert!(agent_string_is_stale_cursor_sdk_auth("Authentication"));
    assert!(agent_string_is_stale_cursor_sdk_auth("ERROR_NOT_LOGGED_IN"));
    assert!(!agent_string_is_stale_cursor_sdk_auth("request timed out"));
}
