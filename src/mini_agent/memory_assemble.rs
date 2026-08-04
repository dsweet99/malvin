//! Sticky Header text and soft-cap helpers for mini chat-state memory.

use crate::mini_agent::protocol::{
    assemble_completion_messages, AssembleInput, SECTION_SHAPE_NUDGE,
};

/// Soft cap for durable chat-state History (chars).
pub const HISTORY_SOFT_CAP: usize = 12_000;
/// Soft cap for Previous RESPONSE body (chars).
pub const PREVIOUS_SOFT_CAP: usize = 8_000;

pub const PREVIOUS_TRUNCATED_MARKER: &str = "\n…[previous response truncated]…\n";

/// Build sticky System Header: constraints, memory schema, section format, model slug.
#[must_use]
pub fn build_sticky_header(mini_constraints: &str, llm_model_slug: &str) -> String {
    let mut parts = Vec::new();
    if !mini_constraints.is_empty() {
        parts.push(mini_constraints.to_string());
    }
    parts.push(MEMORY_SCHEMA.to_string());
    if !llm_model_slug.is_empty() {
        parts.push(format!(
            "Your model slug is `{llm_model_slug}`. When asked which LLM you are, name this slug."
        ));
    }
    parts.join("\n\n")
}

const MEMORY_SCHEMA: &str = r#"## Mini chat-state memory (not workflow log-file History)

Each reply MUST use this exact section order:

## NEW_HISTORY
<replacement chat-state History>

## RESPONSE
<body that answers the New request; when History or the New request requires MALVIN_DM_START/END, put the user-visible answer inside that closed DM fence>

Chat-state History is a compact durable summary (not a full chat transcript). Preserve:
objectives and constraints (including any required DM fence); verified observations; hypotheses with confidence; decisions and reasons;
completed actions and results; failed approaches; unresolved questions; next actions; pointers to
authoritative logs, files, or commits (prefer pointers over inlining large bodies).

Distinguish fact kinds explicitly: observed; user-provided; inference; proposal; verified action.

Previous response (when present on the wire) is the last RESPONSE body only, verbatim short-term memory.
New request is the latest user text, bash observation, or gate-retry divergence note.
Do not confuse chat-state History with workflow header.md "History" (reading run log files)."#;

#[must_use]
pub fn soft_cap_history(history: &str) -> String {
    if history.len() <= HISTORY_SOFT_CAP {
        return history.to_string();
    }
    mid_truncate(history, HISTORY_SOFT_CAP, "\n…[history truncated; compress further next turn]…\n")
}

#[must_use]
pub fn soft_cap_previous(previous: &str) -> String {
    if previous.len() <= PREVIOUS_SOFT_CAP {
        return previous.to_string();
    }
    mid_truncate(previous, PREVIOUS_SOFT_CAP, PREVIOUS_TRUNCATED_MARKER)
}

fn mid_truncate(text: &str, cap: usize, marker: &str) -> String {
    let budget = cap.saturating_sub(marker.len());
    let head = budget / 2;
    let tail = budget.saturating_sub(head);
    let head_end = floor_char_boundary(text, head.min(text.len()));
    let tail_start = ceil_char_boundary(text, text.len().saturating_sub(tail));
    if head_end >= tail_start {
        let end = floor_char_boundary(text, budget.min(text.len()));
        return format!("{}{}", &text[..end], marker);
    }
    format!("{}{}{}", &text[..head_end], marker, &text[tail_start..])
}

/// Largest char boundary index `<= i` (or `text.len()` when `i` is past the end).
#[must_use]
pub(crate) const fn floor_char_boundary(text: &str, mut i: usize) -> usize {
    if i >= text.len() {
        return text.len();
    }
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Smallest char boundary index `>= i` (or `text.len()` when `i` is past the end).
#[must_use]
pub(crate) const fn ceil_char_boundary(text: &str, mut i: usize) -> usize {
    if i >= text.len() {
        return text.len();
    }
    while i < text.len() && !text.is_char_boundary(i) {
        i += 1;
    }
    i
}

pub struct SessionAssemble<'a> {
    pub header: &'a str,
    pub study_act_cue: Option<&'a str>,
    pub history: &'a str,
    pub previous_response: &'a str,
    pub new_request: &'a str,
    pub section_nudge: bool,
}

/// Assemble ephemeral completion messages for one consolidate call.
#[must_use]
pub fn assemble_session_messages(input: SessionAssemble<'_>) -> Vec<crate::openrouter_transport::ChatMessage> {
    let hist = soft_cap_history(input.history);
    let prev = soft_cap_previous(input.previous_response);
    let header_owned = if input.section_nudge {
        format!("{}\n\n{SECTION_SHAPE_NUDGE}", input.header)
    } else {
        input.header.to_string()
    };
    assemble_completion_messages(AssembleInput {
        header: &header_owned,
        study_act_cue: input.study_act_cue,
        history: &hist,
        previous_response: &prev,
        new_request: input.new_request,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openrouter_transport::ChatRole;

    #[test]
    fn soft_cap_history_inserts_compress_marker() {
        let big = "x".repeat(HISTORY_SOFT_CAP + 100);
        let out = soft_cap_history(&big);
        assert!(out.contains("compress further"));
        assert!(out.len() < big.len());
    }

    #[test]
    fn mid_truncate_respects_utf8_char_boundaries() {
        // Each emoji is 4 bytes; an odd byte cut would panic without boundary flooring.
        let text = "😀".repeat(40);
        let out = mid_truncate(&text, 50, "…");
        assert!(out.contains('…'));
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn mid_truncate_fallback_when_head_meets_tail() {
        let text = "😀😀😀";
        let out = mid_truncate(text, 3, "…MARK…");
        assert!(out.contains("MARK"));
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn char_boundary_helpers_cover_mid_codepoint_and_past_end() {
        let s = "😀a";
        assert_eq!(floor_char_boundary(s, 0), 0);
        assert_eq!(floor_char_boundary(s, 1), 0);
        assert_eq!(floor_char_boundary(s, 2), 0);
        assert_eq!(floor_char_boundary(s, 3), 0);
        assert_eq!(floor_char_boundary(s, 4), 4);
        assert_eq!(floor_char_boundary(s, 5), 5);
        assert_eq!(floor_char_boundary(s, 100), s.len());
        assert_eq!(ceil_char_boundary(s, 0), 0);
        assert_eq!(ceil_char_boundary(s, 1), 4);
        assert_eq!(ceil_char_boundary(s, 2), 4);
        assert_eq!(ceil_char_boundary(s, 4), 4);
        assert_eq!(ceil_char_boundary(s, 5), 5);
        assert_eq!(ceil_char_boundary(s, 100), s.len());
    }

    #[test]
    fn assemble_omits_empty_blocks() {
        let msgs = assemble_session_messages(SessionAssemble {
            header: "H",
            study_act_cue: None,
            history: "",
            previous_response: "",
            new_request: "go",
            section_nudge: false,
        });
        assert!(msgs.iter().all(|m| !m.content.contains("Chat-state History")));
        assert_eq!(msgs.last().unwrap().content, "go");
        assert!(matches!(msgs[0].role, ChatRole::System));
    }
}
