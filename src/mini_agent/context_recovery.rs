//! Loop-owned context recovery: soft-cap History first, then Previous.

use crate::mini_agent::loop_driver::LoopDriverSession;
use crate::mini_agent::memory_assemble::{
    soft_cap_history, soft_cap_previous, HISTORY_SOFT_CAP, PREVIOUS_TRUNCATED_MARKER,
};

pub const DROP_STRATEGY_HISTORY_FIRST: &str = "history_first_soft_cap";

pub struct ShrinkEvent {
    pub attempt: u32,
    pub messages_before: usize,
    pub messages_after: usize,
    pub bytes_removed: usize,
}

const fn bytes(session: &LoopDriverSession) -> usize {
    session.history.len() + session.previous_response.len()
}

const fn shrink_event(attempt: u32, before: usize, after: usize) -> ShrinkEvent {
    ShrinkEvent {
        attempt,
        messages_before: 2,
        messages_after: 2,
        bytes_removed: before.saturating_sub(after),
    }
}

/// Soft-cap History; if unchanged, truncate Previous with marker.
#[must_use]
pub fn shrink_session_memory(session: &mut LoopDriverSession, attempt: u32) -> Option<ShrinkEvent> {
    let before = bytes(session);
    let hist = soft_cap_history(&session.history);
    if hist != session.history {
        session.history = hist;
        return Some(shrink_event(attempt, before, bytes(session)));
    }
    let prev = soft_cap_previous(&session.previous_response);
    if prev != session.previous_response {
        session.previous_response = prev;
        return Some(shrink_event(attempt, before, bytes(session)));
    }
    force_shrink(session, attempt, before)
}

fn force_shrink(session: &mut LoopDriverSession, attempt: u32, before: usize) -> Option<ShrinkEvent> {
    if session.history.len() > 64 {
        session.history = halve(&session.history, "\n…[history truncated; compress further next turn]…\n");
        return Some(shrink_event(attempt, before, bytes(session)));
    }
    if session.previous_response.len() > 64 {
        session.previous_response = halve(&session.previous_response, PREVIOUS_TRUNCATED_MARKER);
        return Some(shrink_event(attempt, before, bytes(session)));
    }
    let _ = HISTORY_SOFT_CAP;
    None
}

fn halve(text: &str, marker: &str) -> String {
    let keep = (text.len() / 2).max(32);
    let head = keep / 2;
    format!("{}{}{}", &text[..head], marker, &text[text.len() - (keep - head)..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mini_agent::memory_assemble::PREVIOUS_SOFT_CAP;

    fn empty_session() -> LoopDriverSession {
        LoopDriverSession {
            history: String::new(),
            previous_response: String::new(),
            pending_new_request: None,
            cwd: std::env::temp_dir(),
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
            llm_model_slug: String::new(),
            section_shape_nudged: false,
        }
    }

    #[test]
    fn shrink_history_before_previous() {
        let mut session = empty_session();
        session.history = "h".repeat(HISTORY_SOFT_CAP + 50);
        session.previous_response = "p".repeat(100);
        let event = shrink_session_memory(&mut session, 1).expect("shrink");
        assert!(event.bytes_removed > 0);
        assert!(session.history.contains("compress further"));
    }

    #[test]
    fn shrink_previous_when_history_already_small() {
        let mut session = empty_session();
        session.history = "ok".into();
        session.previous_response = "p".repeat(PREVIOUS_SOFT_CAP + 80);
        let event = shrink_session_memory(&mut session, 1).expect("shrink");
        assert!(event.bytes_removed > 0);
        assert!(session.previous_response.contains("previous response truncated"));
    }
}
