//! Review-chat LGTM acceptance for `malvin explain`.

use crate::review_sync::is_lgtm_str;

/// Review chat acceptance for explain.
///
/// Cursor streams intermediate `agent_message_chunk` prose before the deliverable. Chunks are often
/// concatenated without newlines, so the captured chat can end with `.LGTM` glued onto the prior
/// sentence rather than a bare `LGTM` line.
///
/// A failure-focused gap list must never count as LGTM, even when a trailing `LGTM` is streamed
/// after bullets (gap list + LGTM is not acceptance).
#[must_use]
pub(crate) fn explain_review_chat_is_lgtm(chat: &str) -> bool {
    if is_lgtm_str(chat) {
        return true;
    }
    if explain_review_chat_has_gap_bullets(chat) {
        return false;
    }
    if chat
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .is_some_and(|line| {
            let line = line.strip_prefix('\u{FEFF}').unwrap_or(line).trim();
            line == "LGTM"
        })
    {
        return true;
    }
    let t = chat.trim();
    let t = t.strip_prefix('\u{FEFF}').unwrap_or(t).trim();
    // Streamed final deliverable glued onto the previous sentence (no separating newline).
    t.strip_suffix("LGTM").is_some_and(|prefix| {
        prefix
            .as_bytes()
            .last()
            .is_some_and(|b| matches!(b, b'.' | b'!' | b'?'))
    })
}

#[must_use]
fn explain_review_chat_has_gap_bullets(chat: &str) -> bool {
    chat.lines().any(|line| {
        let t = line.trim_start();
        let t = t.strip_prefix('\u{FEFF}').unwrap_or(t);
        t.starts_with("- ") || t.starts_with("* ") || t.starts_with("• ")
    })
}
