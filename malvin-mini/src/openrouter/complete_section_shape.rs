use crate::openrouter::types::{ChatMessage, ChatRole};
use super::super::memory_format::SECTION_SHAPE_NUDGE;

/// Unique to [`SECTION_SHAPE_NUDGE`] (not sticky MEMORY_SCHEMA, which also mentions NEW_HISTORY).
const SECTION_SHAPE_NUDGE_MARKER: &str = "Do not omit either heading";
const SECTION_SHAPE_USER_NUDGE: &str = "Your previous reply omitted the required wire sections. \
Emit ## NEW_HISTORY then ## RESPONSE only, in that order.";
const SECTION_SHAPE_USER_NUDGE_2: &str = "Wire format still wrong. Reply with ONLY these two markdown \
headings in order: ## NEW_HISTORY then ## RESPONSE. No prose before ## NEW_HISTORY.";

pub(super) fn inject_section_shape_nudge(messages: &mut Vec<ChatMessage>) -> bool {
    let already_system_nudged = messages.iter().any(|m| {
        matches!(m.role, ChatRole::System) && m.content.contains(SECTION_SHAPE_NUDGE_MARKER)
    });
    if !already_system_nudged {
        messages.insert(
            0,
            ChatMessage {
                role: ChatRole::System,
                content: SECTION_SHAPE_NUDGE.to_string(),
            },
        );
        return true;
    }
    let already_user_nudge = messages.iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.contains("omitted the required wire sections")
    });
    if !already_user_nudge {
        messages.push(ChatMessage {
            role: ChatRole::User,
            content: SECTION_SHAPE_USER_NUDGE.to_string(),
        });
        return true;
    }
    let already_user_nudge_2 = messages.iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.contains("Wire format still wrong")
    });
    if !already_user_nudge_2 {
        messages.push(ChatMessage {
            role: ChatRole::User,
            content: SECTION_SHAPE_USER_NUDGE_2.to_string(),
        });
        return true;
    }
    // Final budget slot: strip prior user nudges and re-assert once.
    messages.retain(|m| {
        !(matches!(m.role, ChatRole::User)
            && (m.content.contains("omitted the required wire sections")
                || m.content.contains("Wire format still wrong")))
    });
    messages.push(ChatMessage {
        role: ChatRole::User,
        content: SECTION_SHAPE_USER_NUDGE_2.to_string(),
    });
    true
}
