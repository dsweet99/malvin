//! When a coder session must be torn down before the next retry attempt.

/// Idle Cursor SDK connections can surface as Authentication / `ERROR_NOT_LOGGED_IN`
/// even when the API key is valid (stale gRPC / short-lived token). Evict + resume.
#[must_use]
pub(crate) fn agent_string_is_stale_cursor_sdk_auth(msg: &str) -> bool {
    let text = msg.to_ascii_lowercase();
    if text.contains("authenticationerror") || text == "authentication" {
        return true;
    }
    if text.starts_with("authentication") {
        return true;
    }
    if text.contains("error_not_logged_in") {
        return true;
    }
    if text.contains("[unauthenticated]") || text.contains("unauthenticated") {
        return true;
    }
    text.contains("logged in") && text.contains("logging out")
}

/// Child-health / transport failures where the open coder session must be torn down before retry.
#[must_use]
pub(crate) fn agent_error_requires_coder_session_teardown(msg: &str) -> bool {
    let text = msg.to_ascii_lowercase();
    let child_dead = text.contains("acp child process appears hung")
        || text.contains("acp child process is not running")
        || text.contains("acp child process is zombie")
        || text.contains("acp stdout closed")
        || text.contains("bridge stdout closed")
        || text.contains("bridge write:")
        || text.contains("bridge flush:")
        || text.contains("bridge read:")
        || text.contains("iterable is closed")
        || text.contains("connection stalled");
    if child_dead {
        return true;
    }
    if agent_string_is_stale_cursor_sdk_auth(msg) {
        return true;
    }
    if !crate::acp::agent_string_is_cursor_http2_transport_error(msg) {
        return false;
    }
    // Mock ACP agents stream PING/CANCEL as text but stay healthy; keep the session for retry.
    if crate::acp::test_no_real_agent_enabled() {
        return false;
    }
    true
}
