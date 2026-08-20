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
    "bridge timed out",
    "pi rpc drain timed out",
    "pi rpc timed out",
    "pi rpc stdout closed",
    "pi rpc write:",
    "pi rpc flush:",
    "pi rpc read:",
    "codex timed out",
    "codex stdout closed",
    "codex write:",
    "codex flush:",
    "codex read:",
    "codex json-rpc parse:",
    "currently streaming",
    "iterable is closed",
    "connection stalled",
];

fn text_has_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(n))
}

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
    if crate::acp::test_no_real_agent_enabled() {
        return false;
    }
    true
}
