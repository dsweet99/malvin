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

/// Cursor SDK `AgentBusyError` (409): agent still has an active run (often after hard-kill).
///
/// Resume+send on that agent id fails; the open session must be torn down and the id forgotten
/// so the next attempt uses `Agent.create` instead of resume.
#[must_use]
pub(crate) fn agent_string_is_cursor_agent_busy(msg: &str) -> bool {
    msg.to_ascii_lowercase().contains("already has active run")
}

const CHILD_OR_BRIDGE_DEAD_NEEDLES: &[&str] = &[
    "acp child process appears hung",
    "acp child process is not running",
    "acp child process is zombie",
    "acp stdout closed",
    "bridge stdout closed",
    "bridge write:",
    "bridge flush:",
    "bridge read:",
    "bridge drain timed out",
    "pi rpc drain timed out",
    "pi rpc stdout closed",
    "pi rpc write:",
    "pi rpc flush:",
    "pi rpc read:",
    "currently streaming",
    "iterable is closed",
    "connection stalled",
];

fn text_has_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(n))
}

/// Child-health / transport failures where the open coder session must be torn down before retry.
#[must_use]
pub(crate) fn agent_error_requires_coder_session_teardown(msg: &str) -> bool {
    let text = msg.to_ascii_lowercase();
    if text_has_any(&text, CHILD_OR_BRIDGE_DEAD_NEEDLES) {
        return true;
    }
    if agent_string_is_cursor_agent_busy(msg) {
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
