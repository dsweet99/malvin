use crate::openrouter::memory_format::CHAT_STATE_HISTORY_LABEL;
use crate::openrouter::types::{ChatMessage, ChatRole};

/// Drop or truncate messages so a local retry can fit under a prompt-token budget.
/// Never deletes leading Header System messages. Prefers shrinking Chat-state History.
pub(crate) fn shrink_prompt_messages(messages: &mut Vec<ChatMessage>) -> bool {
    if let Some(idx) = history_system_index(messages)
        && truncate_message_content(messages, idx)
    {
        return true;
    }
    shrink_non_system(messages)
}

fn history_system_index(messages: &[ChatMessage]) -> Option<usize> {
    messages.iter().position(|m| {
        matches!(m.role, ChatRole::System) && m.content.contains(CHAT_STATE_HISTORY_LABEL)
    })
}

fn shrink_non_system(messages: &mut Vec<ChatMessage>) -> bool {
    let idxs: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| !matches!(m.role, ChatRole::System))
        .map(|(i, _)| i)
        .collect();
    if idxs.is_empty() {
        return false;
    }
    if let Some(&idx) = idxs
        .iter()
        .find(|&&i| matches!(messages[i].role, ChatRole::Assistant))
        && truncate_message_content(messages, idx)
    {
        return true;
    }
    if idxs.len() > 1 {
        let last = *idxs.last().unwrap();
        let drop_i = idxs.iter().copied().find(|&i| i != last).unwrap_or(idxs[0]);
        messages.remove(drop_i);
        return true;
    }
    truncate_message_content(messages, idxs[0])
}

fn truncate_message_content(messages: &mut [ChatMessage], idx: usize) -> bool {
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
