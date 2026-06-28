//! Loop-owned context recovery: replace oldest whole messages with placeholders on overflow.

use malvin_mini::ChatRole;

use crate::agent_backend::mini::loop_driver::LoopDriverSession;

pub const DROP_STRATEGY_OLDEST_WHOLE: &str = "drop_oldest_whole_message";

const REMOVED_MESSAGE_PREFIX: &str = "[Message removed to save context space.";

#[must_use]
pub fn removed_message_placeholder(bytes: usize) -> String {
    format!("[Message removed to save context space. {bytes} bytes].")
}

fn is_removed_message_placeholder(content: &str) -> bool {
    content.starts_with(REMOVED_MESSAGE_PREFIX)
}

/// Replace the oldest non-system message with a short placeholder, or remove it if
/// already replaced. Returns false when nothing removable.
#[must_use]
pub fn drop_oldest_whole_message(session: &mut LoopDriverSession) -> bool {
    let Some(idx) = session
        .messages
        .iter()
        .position(|m| m.role != ChatRole::System)
    else {
        return false;
    };
    if is_removed_message_placeholder(&session.messages[idx].content) {
        session.messages.remove(idx);
        return true;
    }
    let removed_bytes = session.messages[idx].content.len();
    session.messages[idx].content = removed_message_placeholder(removed_bytes);
    true
}

pub struct ShrinkEvent {
    pub attempt: u32,
    pub messages_before: usize,
    pub messages_after: usize,
    pub bytes_removed: usize,
}

/// Drop oldest whole message and return shrink metadata for trace emission.
#[must_use]
pub fn shrink_one_whole_message(session: &mut LoopDriverSession, attempt: u32) -> Option<ShrinkEvent> {
    let before_len = session.messages.len();
    let bytes_before: usize = session.messages.iter().map(|m| m.content.len()).sum();
    if !drop_oldest_whole_message(session) {
        return None;
    }
    let bytes_after: usize = session.messages.iter().map(|m| m.content.len()).sum();
    Some(ShrinkEvent {
        attempt,
        messages_before: before_len,
        messages_after: session.messages.len(),
        bytes_removed: bytes_before.saturating_sub(bytes_after),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use malvin_mini::ChatMessage;

    #[test]
    fn drop_oldest_skips_system_and_replaces_first_user_with_placeholder() {
        let mut session = LoopDriverSession {
            messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: "sys".into(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: "old".into(),
                },
                ChatMessage {
                    role: ChatRole::Assistant,
                    content: "mid".into(),
                },
            ],
            cwd: std::env::temp_dir(),
            constraints_prepended: true,
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
        llm_model_slug: String::new(),
        };
        assert!(drop_oldest_whole_message(&mut session));
        assert_eq!(session.messages.len(), 3);
        assert_eq!(session.messages[0].content, "sys");
        assert_eq!(
            session.messages[1].content,
            removed_message_placeholder("old".len())
        );
        assert_eq!(session.messages[2].content, "mid");
    }

    #[test]
    fn drop_oldest_removes_existing_placeholder_on_next_pass() {
        let placeholder = removed_message_placeholder(100);
        let mut session = LoopDriverSession {
            messages: vec![
                ChatMessage {
                    role: ChatRole::User,
                    content: placeholder.clone(),
                },
                ChatMessage {
                    role: ChatRole::Assistant,
                    content: "keep".into(),
                },
            ],
            cwd: std::env::temp_dir(),
            constraints_prepended: true,
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
        llm_model_slug: String::new(),
        };
        assert!(drop_oldest_whole_message(&mut session));
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "keep");
        let _ = placeholder;
    }

    #[test]
    fn shrink_one_whole_message_returns_event_metadata() {
        let original = "x".repeat(500);
        let mut session = LoopDriverSession {
            messages: vec![
                ChatMessage {
                    role: ChatRole::User,
                    content: original.clone(),
                },
                ChatMessage {
                    role: ChatRole::Assistant,
                    content: "keep".into(),
                },
            ],
            cwd: std::env::temp_dir(),
            constraints_prepended: true,
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
        llm_model_slug: String::new(),
        };
        let event = shrink_one_whole_message(&mut session, 1).expect("shrink");
        assert_eq!(event.attempt, 1);
        assert_eq!(event.messages_before, 2);
        assert_eq!(event.messages_after, 2);
        assert_eq!(
            event.bytes_removed,
            original.len() - removed_message_placeholder(original.len()).len()
        );
        assert_eq!(
            session.messages[0].content,
            removed_message_placeholder(original.len())
        );
    }

    #[test]
    fn shrink_returns_none_when_only_system_message() {
        let mut session = LoopDriverSession {
            messages: vec![ChatMessage {
                role: ChatRole::System,
                content: "sys".into(),
            }],
            cwd: std::env::temp_dir(),
            constraints_prepended: true,
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
        llm_model_slug: String::new(),
        };
        assert!(shrink_one_whole_message(&mut session, 1).is_none());
    }
}
