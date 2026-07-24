use crate::openrouter::types::{ChatMessage, ChatRole};

/// Drop or truncate messages so a local retry can fit under a prompt-token budget.
pub(crate) fn shrink_prompt_messages(messages: &mut Vec<ChatMessage>) -> bool {
    let non_system_idx: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| !matches!(m.role, ChatRole::System))
        .map(|(i, _)| i)
        .collect();
    if non_system_idx.is_empty() {
        return false;
    }
    if non_system_idx.len() > 1 {
        messages.remove(non_system_idx[0]);
        return true;
    }
    let idx = non_system_idx[0];
    let content = &messages[idx].content;
    if content.len() < 64 {
        return false;
    }
    let keep = content.len() / 2;
    let head = keep / 2;
    let tail = keep - head;
    messages[idx].content = format!(
        "{}…[truncated]…{}",
        &content[..head],
        &content[content.len() - tail..]
    );
    true
}
